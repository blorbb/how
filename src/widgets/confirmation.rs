use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    layout::{Flex, Rect},
    style::Stylize,
    widgets::{Block, Clear, Padding, Paragraph, Widget, Wrap},
};
use ratatui_macros::{horizontal, vertical};
use tui_textarea::{Input, Key};

pub struct ConfirmDialog<T> {
    text: String,
    confirm_callback: Box<dyn FnOnce(&mut T) -> Result<()>>,
}

impl<T> ConfirmDialog<T> {
    pub fn new(
        text: impl Into<String>,
        confirm_callback: impl FnOnce(&mut T) -> Result<()> + 'static,
    ) -> Self {
        Self {
            text: text.into(),
            confirm_callback: Box::new(confirm_callback),
        }
    }

    /// Reads an input and returns whether they confirm `Some(true)`,
    /// cancel `Some(false)`, or enter a key that does nothing `None`.
    pub fn read(&self, input: impl Into<Input>) -> Option<bool> {
        match input.into().key {
            Key::Enter => Some(true),
            Key::Char('y') => Some(true),
            Key::Esc => Some(false),
            Key::Char('n') => Some(false),
            _ => None,
        }
    }

    pub fn execute(self, t: &mut T) -> Result<()> {
        (self.confirm_callback)(t)
    }
}

impl<T> Widget for &ConfirmDialog<T> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let text_max_width = area.width - 4;
        let text_height: u16 = self
            .text
            .lines()
            // height of each line, accounting for wrapping
            .map(|l| l.len() as u16 / text_max_width + 1)
            .sum();

        // dialog buttons:
        // ┌────────────┐   ┌─────────────┐
        // │ Cancel (n) │   │ Confirm (y) │
        // └────────────┘   └─────────────┘
        // 32 characters wide, 3 characters tall
        const BUTTONS_WIDTH: u16 = 32;
        const BUTTON_HEIGHT: u16 = 3;
        const PADDING_BLOCK: u16 = 2;
        const PADDING_INLINE: u16 = 3;
        let popup_area = {
            let text_width = self.text.lines().map(str::len).max().unwrap_or(0) as u16;
            let width = text_width.clamp(BUTTONS_WIDTH + PADDING_INLINE * 2, text_max_width);

            let height = text_height + PADDING_BLOCK * 2 + BUTTON_HEIGHT;

            let [vert] = vertical![==height].flex(Flex::Center).areas(area);
            let [hor] = horizontal![==width].flex(Flex::Center).areas(vert);
            hor
        };

        Clear.render(popup_area, buf);
        let block = Block::bordered().red().padding(Padding {
            left: PADDING_INLINE - 1,
            right: PADDING_INLINE - 1,
            top: PADDING_BLOCK - 1,
            bottom: PADDING_BLOCK - 1,
        });
        let [text, buttons] = vertical![==text_height, ==BUTTON_HEIGHT]
            .flex(Flex::SpaceAround)
            .areas(block.inner(popup_area));
        let [text] = horizontal![==text.width]
            .flex(Flex::SpaceAround)
            .areas(text);
        let [buttonl, _, buttonr] = horizontal![==14, ==3, ==15]
            .flex(Flex::Center)
            .areas(buttons);

        block.render(popup_area, buf);
        let text_paragraph = Paragraph::new(&*self.text).wrap(Wrap { trim: false });
        text_paragraph.render(text, buf);

        let cancel_button = Paragraph::new(" Cancel (n) ")
            .block(Block::bordered())
            .red();
        let confirm_button = Paragraph::new(" Confirm (y) ")
            .block(Block::bordered())
            .green();
        cancel_button.render(buttonl, buf);
        confirm_button.render(buttonr, buf);
    }
}
