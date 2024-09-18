use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    widgets::{Block, Borders, Paragraph},
    DefaultTerminal, Frame,
};
use ratatui_macros::{horizontal, vertical};
use tui_textarea::TextArea;

use crate::{
    db::{Data, Entry},
    rank,
};

pub fn run(mut terminal: DefaultTerminal, mut data: Data) -> Result<()> {
    data.add(Entry::new(
        "Test title",
        "Description thing",
        "answer is this",
    ))?;
    data.add(Entry::new(
        "Apples and oranges",
        "This is a description that i have",
        "apple --add thing",
    ))?;
    data.add(Entry::new(
        "Git can do something here",
        "",
        "git diff main..",
    ))?;

    let mut query = TextArea::default();
    query.set_block(Block::default().borders(Borders::ALL).title("Search"));

    loop {
        terminal.draw(|f| draw(f, &query, &rank::rank(text(&query), data.entries())))?;

        if let Event::Key(k) = event::read()? {
            let KeyEventKind::Press = k.kind else {
                continue;
            };

            if k.modifiers.contains(KeyModifiers::CONTROL) && k.code == KeyCode::Char('q') {
                break;
            }

            query.input(k);

            let matches = rank::rank(text(&query), data.entries());

            terminal.draw(|f| draw(f, &query, &matches))?;
        }
    }

    Ok(())
}

fn draw(frame: &mut Frame, query: &TextArea, matches: &[(&Entry, u16)]) {
    let hor = horizontal![==1/2; 2].split(frame.area());
    let vert = vertical![==3, *=1].split(hor[0]);
    let p = Paragraph::new(format!("{matches:#?}"));
    frame.render_widget(query, vert[0]);
    frame.render_widget(p, vert[1]);
}

fn text<'a>(area: &'a TextArea<'a>) -> &'a str {
    &area.lines()[0].trim()
}
