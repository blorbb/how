use std::{cell::RefCell, num::Saturating, rc::Rc};

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    style::Stylize,
    widgets::{Block, Borders, StatefulWidget, Widget},
    DefaultTerminal,
};
use ratatui_macros::{horizontal, line, vertical};
use tui_textarea::TextArea;
use tui_widget_list::{ListBuilder, ListState, ListView};

use crate::{db::Data, rank};

pub struct App {
    data: Rc<RefCell<Data>>,
    query: TextArea<'static>,
    matches: Vec<(usize, f32)>,
    list_index: Saturating<usize>,
}

impl App {
    pub fn new(data: Data) -> Self {
        let mut query = TextArea::default();
        query.set_block(Block::default().borders(Borders::ALL).title("Search"));
        Self {
            matches: rank::rank("", data.entries()),
            data: Rc::new(RefCell::new(data)),
            query,
            list_index: Saturating(0),
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.draw(terminal)?;

        loop {
            if let Event::Key(k) = event::read()? {
                let KeyEventKind::Press = k.kind else {
                    continue;
                };

                if k.modifiers.contains(KeyModifiers::CONTROL) && k.code == KeyCode::Char('q') {
                    break;
                }

                match k.code {
                    KeyCode::Down => self.next_item(),
                    KeyCode::Up => self.prev_item(),
                    _ => self.register_input(k),
                }

                self.draw(terminal)?;
            }
        }

        Ok(())
    }

    fn next_item(&mut self) {
        self.list_index = Saturating((self.list_index.0 + 1).min(self.matches.len() - 1))
    }

    fn prev_item(&mut self) {
        self.list_index -= 1
    }

    fn register_input(&mut self, ev: KeyEvent) {
        self.query.input(ev);
        let borrow = self.data.borrow();
        self.matches = rank::rank(self.query_text(), borrow.entries());
        self.list_index = Saturating(0);
    }

    fn draw(&self, terminal: &mut DefaultTerminal) -> Result<()> {
        terminal.draw(|frame| {
            frame.render_widget(self, frame.area());
        })?;
        Ok(())
    }

    fn query_text(&self) -> &str {
        &self.query.lines()[0].trim()
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let hor = horizontal![==1/2; 2].split(area);
        let vert = vertical![==3, *=1].split(hor[0]);

        let data = Rc::clone(&self.data);
        let matches = self.matches.clone();

        let builder = ListBuilder::new(move |cx| {
            let item = Rc::clone(&data.borrow().entries()[matches[cx.index].0]);
            let title = line![
                item.title().to_string(),
                format!(" ({:.4})", matches[cx.index].1)
            ];
            let title = if cx.is_selected {
                title.on_gray()
            } else {
                title
            };

            (title, 1)
        });
        let list = ListView::new(builder, self.matches.len().min(50));

        let mut list_state = ListState::default();
        list_state.select(Some(self.list_index.0));

        self.query.render(vert[0], buf);
        list.render(vert[1], buf, &mut list_state);

        let binding = self.data.borrow();
        let selected = &binding.entries()[self.matches[self.list_index.0].0];
        selected.render(hor[1], buf);
    }
}
