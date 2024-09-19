use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Widget},
};
use tui_textarea::{CursorMove, Input, Key, TextArea as TuiTextArea};

use crate::db::Entry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Wrapping<const SIZE: u32>(u32);

impl<const SIZE: u32> Wrapping<SIZE> {
    pub fn new(num: u32) -> Self {
        Self(num)
    }

    pub fn get(self) -> u32 {
        self.0
    }

    pub fn next(&mut self) {
        self.0 = (self.get() + 1) % SIZE;
    }

    pub fn prev(&mut self) {
        if self.get() == 0 {
            self.0 = SIZE - 1;
        } else {
            self.0 -= 1;
        }
    }
}

impl<const SIZE: u32> PartialEq<u32> for Wrapping<SIZE> {
    fn eq(&self, other: &u32) -> bool {
        self.get() == *other
    }
}

impl<const SIZE: u32> Default for Wrapping<SIZE> {
    fn default() -> Self {
        Self::new(0)
    }
}

#[must_use]
pub enum Action {
    Exit,
    AddEntry(Entry),
}

/// A wrapper around `tui_textarea`'s `TextArea` struct.
pub struct TextArea {
    inner: TuiTextArea<'static>,
    single_line: bool,
}

impl TextArea {
    pub fn new_blurred(initial: impl Into<String>, title: &'static str) -> Self {
        let mut ta = TuiTextArea::from(initial.into().lines());
        ta.set_block(Block::bordered().title(title));
        ta.move_cursor(CursorMove::End);

        let mut this = Self {
            inner: ta,
            single_line: false,
        };
        this.blur();
        this
    }

    pub fn new_focused(initial: impl Into<String>, title: &'static str) -> Self {
        let mut this = Self::new_blurred(initial, title);
        this.focus();
        this
    }

    pub fn set_single_line(mut self) -> Self {
        self.single_line = true;
        self
    }

    pub fn update_block(&mut self, f: impl FnOnce(Block<'static>) -> Block<'static>) {
        let old_block = self.inner.block().unwrap().clone();
        let new_block = f(old_block);
        self.inner.set_block(new_block);
    }

    pub fn focus(&mut self) {
        self.update_block(|b| b.border_style(Color::LightYellow));
        self.inner
            .set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
    }

    pub fn blur(&mut self) {
        self.update_block(|b| b.border_style(Color::White));
        self.inner.set_cursor_style(Style::default());
    }

    pub fn input(&mut self, input: impl Into<Input>) {
        let input: Input = input.into();
        match input {
            // prevent newline with the default keyboard shortcuts
            Input {
                key: Key::Enter, ..
            }
            | Input {
                key: Key::Char('m'),
                ctrl: true,
                ..
            } if self.single_line => {}
            _ => drop(self.inner.input(input)),
        }
    }

    // regular delegated methods //

    pub fn lines(&self) -> &[String] {
        self.inner.lines()
    }

    pub fn text(&self) -> String {
        self.lines().join("\n")
    }
}

impl Widget for &TextArea {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.inner.render(area, buf);
    }
}
