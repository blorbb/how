use ratatui::{
    buffer::Buffer, layout::Rect, style::{Color, Modifier, Style}, widgets::{Block, Borders, Widget}
};
use tui_textarea::{CursorMove, Input, TextArea as TuiTextArea};

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
pub struct TextArea(TuiTextArea<'static>);

impl TextArea {
    pub fn new_blurred(initial: impl Into<String>, title: &'static str) -> Self {
        let mut ta = TuiTextArea::from(initial.into().lines());
        ta.set_block(Block::default().borders(Borders::ALL).title(title));
        ta.move_cursor(CursorMove::End);

        let mut this = Self(ta);
        this.blur();
        this
    }

    pub fn new_focused(initial: impl Into<String>, title: &'static str) -> Self {
        let mut this = Self::new_blurred(initial, title);
        this.focus();
        this
    }

    pub fn update_block(&mut self, f: impl FnOnce(Block<'static>) -> Block<'static>) {
        let old_block = self.0.block().unwrap().clone();
        let new_block = f(old_block);
        self.0.set_block(new_block);
    }

    pub fn focus(&mut self) {
        self.update_block(|b| b.border_style(Color::LightYellow));
        self.0.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
    }

    pub fn blur(&mut self) {
        self.update_block(|b| b.border_style(Color::White));
        self.0.set_cursor_style(Style::default());
    }

    // regular delegated methods //

    pub fn input(&mut self, input: impl Into<Input>) -> bool {
        self.0.input(input)
    }

    pub fn lines(&self) -> &[String] {
        self.0.lines()
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
        self.0.render(area, buf);
    }
}
