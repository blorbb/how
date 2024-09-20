mod db;
mod rank;
mod ui;
mod utils;
mod widgets;

use std::fs;

use color_eyre::{
    eyre::{Context, ContextCompat},
    Result,
};
use crossterm::event::{self, Event, KeyEventKind};
use db::Data;
use ui::App;
fn main() -> Result<()> {
    let dir = dirs::data_dir().context("unable to find data directory")?;

    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(dir.join("how-db.toml"))
        .context("unable to open how-db.toml")?;

    let mut terminal = ratatui::init();
    terminal.clear()?;

    let data = Data::load_from(file)?;
    let mut app = App::new(data);
    let output = loop {
        terminal.draw(|f| f.render_widget(&app, f.area()))?;
        if let Event::Key(input) = event::read()? {
            if input.kind == KeyEventKind::Release {
                continue;
            }
            match app.read(input.into())? {
                ui::AppControl::Become(s) => break Some(s),
                ui::AppControl::Exit => break None,
                ui::AppControl::Continue => {}
            }
        }
    };

    ratatui::restore();

    if let Some(s) = output {
        println!("{s}");
    }

    Ok(())
}
