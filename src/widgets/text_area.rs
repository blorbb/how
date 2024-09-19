use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Widget},
};
use tui_textarea::{CursorMove, Input, Key, TextArea as TuiTextArea};

const FOCUSED_COLOR: Color = Color::LightYellow;
const BLURRED_COLOR: Color = Color::White;
const ERROR_COLOR: Color = Color::Red;

/// A wrapper around `tui_textarea`'s `TextArea` struct.
pub struct TextArea {
    inner: TuiTextArea<'static>,
    single_line: bool,
    title: &'static str,
    focused: bool,
    validator: Option<(&'static str, Box<dyn Fn(&'_ str) -> bool>)>,
}

impl TextArea {
    pub fn new_blurred(initial: impl Into<String>, title: &'static str) -> Self {
        let mut ta = TuiTextArea::from(initial.into().lines());
        ta.set_block(Block::bordered().title(title));
        ta.move_cursor(CursorMove::End);

        let mut this = Self {
            inner: ta,
            single_line: false,
            title,
            focused: false,
            validator: None,
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

    pub fn set_validator(
        mut self,
        error_msg: &'static str,
        validator: impl Fn(&str) -> bool + 'static,
    ) -> Self {
        self.validator = Some((error_msg, Box::new(validator)));
        self.update_validation();
        self
    }

    fn update_validation(&mut self) {
        if let Some((msg, validator)) = &self.validator {
            if !validator(&self.text()) {
                self.set_title(msg);
                self.color_border(ERROR_COLOR);
            } else {
                self.set_title(self.title);
                self.color_border(self.border_color());
            }
        }
    }

    pub fn is_valid(&self) -> bool {
        !self
            .validator
            .as_ref()
            .is_some_and(|(_, val)| !val(&self.text()))
    }

    fn update_block(&mut self, f: impl FnOnce(Block<'static>) -> Block<'static>) {
        let old_block = self.inner.block().unwrap().clone();
        let new_block = f(old_block);
        self.inner.set_block(new_block);
    }

    pub fn color_border(&mut self, color: Color) {
        self.update_block(|b| b.border_style(color));
    }

    pub fn set_title(&mut self, title: &'static str) {
        // .title appends a new title instead of replacing :(
        self.inner.set_block(
            Block::bordered()
                .border_style(self.border_color())
                .title(title),
        )
    }

    pub fn focus(&mut self) {
        self.focused = true;
        let color = self.border_color();
        self.update_block(|b| b.border_style(color));
        self.inner
            .set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
    }

    pub fn blur(&mut self) {
        self.focused = false;
        let color = self.border_color();
        self.update_block(|b| b.border_style(color));
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
        self.update_validation();
    }

    // regular delegated methods //

    pub fn lines(&self) -> &[String] {
        self.inner.lines()
    }

    pub fn text(&self) -> String {
        self.lines().join("\n")
    }

    fn border_color(&self) -> Color {
        if !self.is_valid() {
            ERROR_COLOR
        } else if self.focused {
            FOCUSED_COLOR
        } else {
            BLURRED_COLOR
        }
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
