use std::{cell::RefCell, num::Saturating, rc::Rc};

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use itertools::Itertools;
use ratatui::{
    widgets::{Block, Borders, Paragraph, Widget},
    DefaultTerminal,
};
use ratatui_macros::{horizontal, vertical};
use tui_textarea::TextArea;

use crate::{
    db::{Data, Entry},
    rank,
};

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
            data: Rc::new(RefCell::new(data)),
            query,
            matches: Vec::new(),
            list_index: Saturating(0),
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.data.borrow_mut().add(Entry::new(
            "Test title",
            "Description thing",
            "answer is this",
        ))?;
        self.data.borrow_mut().add(Entry::new(
            "Apples and oranges",
            "This is a description that i have",
            "apple --add thing",
        ))?;
        self.data.borrow_mut().add(Entry::new(
            "Git can do something here",
            "",
            "git diff main..",
        ))?;

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
                    KeyCode::Down => self.list_index += 1,
                    KeyCode::Up => self.list_index -= 1,
                    _ => _ = self.query.input(k),
                }

                let borrow = self.data.borrow();
                self.matches = rank::rank(self.query_text(), borrow.entries());
                self.draw(terminal)?;
            }
        }

        Ok(())
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

        let matches = self
            .matches
            .iter()
            .map(|(i, rank)| (Rc::clone(&self.data.borrow().entries()[*i]), rank))
            .collect_vec();
        let p = Paragraph::new(format!("{matches:#?}"));
        self.query.render(vert[0], buf);
        p.render(vert[1], buf);
    }
}
