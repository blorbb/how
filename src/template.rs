//! Parsing for a code snippet template.
//!
//! Snippet is generally of the form:
//! ```sh
//! git diff [main#from]..[#to]
//! ```
//! - Templates are wrapped in brackets.
//! - Contains a default before the #.
//! - Contains a description after the #.
//! - [] and # can be escaped with `\`. Escapes can be re-escaped like `\\`
//!   for a literal backslash. This is only needed if there needs to be a
//!   backslash before a `[`, `]`, or `#` for some reason - all other
//!   backslashes that are followed by any other character will be treated
//!   as a literal backslash.
//!
//! A second hash will also add an index (starting from 1):
//! ```sh
//! git diff {main#from#1}..{#to#2}
//! ```
//! - Can contain an optional number after the # for the tab order.
//!   If no `#num` is provided, it will just go left to right, filling
//!   in gaps between numbers. For example:
//!   ```sh
//!   some-command {a} {b##3} {c} {d}
//!   ```
//!   Will cycle in the order `a`, `c`, `b`, `d`.
//! - Multiple templates can have the same number, in which case both will
//!   be selected and edited at the same time.

use std::{collections::HashMap, mem, ops::Range};

use ir::{IncrementalU8, State};
use itertools::Itertools;
use thiserror::Error;

mod ir {
    use std::ops::Range;

    use super::Error;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct IncrementalU8(u8);

    impl IncrementalU8 {
        pub fn new() -> Self {
            Self(0)
        }

        pub fn read_digit(self, c: char) -> Result<Self, Error> {
            let digit = c
                .to_digit(10)
                .ok_or(Error::InvalidNumber)?
                .try_into()
                .expect("digit is in base 10, value should not exceed 9");
            Ok(Self(
                self.0
                    .checked_mul(10)
                    .and_then(|i| i.checked_add(digit))
                    .ok_or(Error::OverflowingNumber)?,
            ))
        }

        pub fn get(self) -> u8 {
            self.0
        }
    }

    impl From<u8> for IncrementalU8 {
        fn from(value: u8) -> Self {
            Self(value)
        }
    }

    // usizes are the range in the **display** string that the default
    // value will be at.
    pub enum State {
        /// Where the literal started
        Literal(usize),
        /// Where the default value of the input started
        Default(usize),
        /// Range of the default text in the *display* text;
        /// Description.
        Description(Range<usize>, String),
        /// Range of the default text;
        /// Description;
        /// Optional index (only `None` when no number provided).
        Index(Range<usize>, String, Option<IncrementalU8>),
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("found bracket within bracket: escape at least one to clarify intent")]
    BracketInBracket,
    #[error("unbalanced bracket templates: escape braces that are to be treated as literals")]
    UnbalancedBrackets,
    #[error("invalid input index: must be a positive integer")]
    InvalidNumber,
    #[error("invalid input index: number too large")]
    OverflowingNumber,
    #[error("no input index given: remove the second '#' for automatic index")]
    MissingNumber,
    #[error("too many hashes in input: escape #'s that are to be treated as literals")]
    TooManyFields,
}

fn parse(s: &str) -> Result<TemplatedCommand, Error> {
    let mut is_escaped = false;
    // buffer characters to push after escapes are handled
    let mut to_push = None;

    let mut input_state = State::Literal(0);

    let mut unassigned_inputs = Vec::<usize>::new();
    let mut input_indexes = HashMap::<IncrementalU8, Vec<usize>>::new();
    let mut template = TemplatedCommand::default();

    for c in s.chars() {
        match (c, &mut input_state) {
            // handle escape characters
            ('[' | ']' | '#' | '\\', _) if is_escaped => {
                is_escaped = false;
                to_push = Some(c);
            }
            ('\\', _) => {
                is_escaped = true;
                continue;
            }

            // starting a new input field with {
            ('[', State::Literal(start)) => {
                let literal_range = *start..template.display.len();
                if !literal_range.is_empty() {
                    template.push_literal(literal_range);
                };
                input_state = State::Default(template.display.len());
            }
            ('[', _) => return Err(Error::BracketInBracket),
            // read first #
            ('#', State::Default(start)) => {
                input_state = State::Description(*start..template.display.len(), String::new())
            }
            // read second #
            ('#', State::Description(range, desc)) => {
                input_state = State::Index(range.clone(), mem::take(desc), None)
            }
            // error on third #
            ('#', State::Index(..)) => return Err(Error::TooManyFields),
            // reading # on literal is fine

            // closing input
            (']', State::Literal(..)) => return Err(Error::UnbalancedBrackets),
            // only default value
            (']', State::Default(start)) => {
                let range = *start..template.display.len();
                unassigned_inputs.push(template.sections.len());
                template.push_input(range, String::new());
                input_state = State::Literal(template.display.len());
            }
            // description given, unassigned ordering
            (']', State::Description(range, desc)) => {
                unassigned_inputs.push(template.sections.len());
                template.push_input(range.clone(), mem::take(desc));
                input_state = State::Literal(template.display.len());
            }
            // index given
            // ensure there is a number assigned, error on inputs like `[a#b#]`
            (']', State::Index(_, _, None)) => return Err(Error::InvalidNumber),
            // fail on 0 index as well
            (']', State::Index(_, _, Some(idx))) if idx.get() == 0 => {
                return Err(Error::MissingNumber)
            }
            (']', State::Index(range, desc, Some(idx))) => {
                input_indexes
                    .entry(*idx)
                    .or_default()
                    .push(template.sections.len());
                template.push_input(range.clone(), mem::take(desc));
                input_state = State::Literal(template.display.len());
            }

            // displayed text: literal or default value
            (_, State::Literal(..) | State::Default(..)) => to_push = Some(c),
            // read index of input field
            (_, State::Index(_, _, ref mut idx)) => {
                *idx = Some(idx.unwrap_or_default().read_digit(c)?)
            }
            // read description of input field
            (_, State::Description(.., ref mut s)) => s.push(c),
        }

        // prev character was a `\`, did not escape anything
        if is_escaped {
            template.display.push('\\');
            is_escaped = false;
        }
        template.display.extend(to_push.take());
    }

    // reverse for efficient popping from the left
    unassigned_inputs.reverse();

    // fill any gaps in input indexes
    for (input_order, section_indexes) in input_indexes
        .into_iter()
        .map(|(a, b)| (usize::from(a.get()), b))
        .sorted()
    {
        // once input_order is equal what's stored in the template,
        // push that on as the next item.
        while input_order < template.input_order.len()
            && let Some(unassigned_input) = unassigned_inputs.pop()
        {
            template.input_order.push(vec![unassigned_input]);
        }
        template.input_order.push(section_indexes);
    }

    // add the remaining unassigned inputs
    unassigned_inputs.reverse();
    template
        .input_order
        .extend(unassigned_inputs.into_iter().map(|i| vec![i]));

    Ok(template)
}

#[derive(Debug)]
pub enum TemplateSection {
    Literal(Range<usize>),
    Input(Range<usize>, String),
}

#[derive(Debug, Default)]
pub struct TemplatedCommand {
    display: Vec<char>,
    sections: Vec<TemplateSection>,
    /// Order in which to jump to an input.
    ///
    /// Numbers are the indices of the input `sections`. Each index
    /// must correspond to a [`TemplateSection::Input`] variant.
    input_order: Vec<Vec<usize>>,
}

impl TemplatedCommand {
    pub fn push_input(&mut self, range: Range<usize>, description: String) {
        self.sections
            .push(TemplateSection::Input(range, description));
    }

    pub fn push_literal(&mut self, range: Range<usize>) {
        self.sections.push(TemplateSection::Literal(range));
    }
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    pub fn works() {
        _ = dbg!(parse("git diff [main#from#1]..[#to]"));
    }
}
