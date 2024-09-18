use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    widgets::{Block, Borders, Paragraph},
    DefaultTerminal,
};
use ratatui_macros::{horizontal, vertical};
use tui_textarea::TextArea;

use crate::{
    db::{Data, Entry},
    rank,
};

pub struct App {
    data: Data,
    query: TextArea<'static>,
}

impl App {
    pub fn new(data: Data) -> Self {
        let mut query = TextArea::default();
        query.set_block(Block::default().borders(Borders::ALL).title("Search"));
        Self { data, query }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.data.add(Entry::new(
            "Test title",
            "Description thing",
            "answer is this",
        ))?;
        self.data.add(Entry::new(
            "Apples and oranges",
            "This is a description that i have",
            "apple --add thing",
        ))?;
        self.data.add(Entry::new(
            "Git can do something here",
            "",
            "git diff main..",
        ))?;

        self.draw(terminal, &[])?;

        loop {
            if let Event::Key(k) = event::read()? {
                let KeyEventKind::Press = k.kind else {
                    continue;
                };

                if k.modifiers.contains(KeyModifiers::CONTROL) && k.code == KeyCode::Char('q') {
                    break;
                }

                self.query.input(k);
                let matches = rank::rank(self.query_text(), self.data.entries());

                self.draw(terminal, &matches)?;
            }
        }

        Ok(())
    }

    fn draw(&self, terminal: &mut DefaultTerminal, matches: &[(&Entry, f32)]) -> Result<()> {
        terminal.draw(|frame| {
            let hor = horizontal![==1/2; 2].split(frame.area());
            let vert = vertical![==3, *=1].split(hor[0]);
            let p = Paragraph::new(format!("{matches:#?}"));
            frame.render_widget(&self.query, vert[0]);
            frame.render_widget(p, vert[1]);
        })?;
        Ok(())
    }

    fn query_text(&self) -> &str {
        &self.query.lines()[0].trim()
    }
}
