mod db;
mod rank;
mod ui;
mod utils;
mod widgets;

use std::{
    fs,
    io::{self, stderr, BufWriter},
};

use color_eyre::{
    eyre::{Context, ContextCompat},
    Result,
};
use crossterm::{
    event::{self, Event, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use db::Data;
use ratatui::{prelude::CrosstermBackend, Terminal};
use ui::App;

fn main() -> Result<()> {
    let dir = dirs::data_dir().context("unable to find data directory")?;

    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(dir.join("how-db.toml"))
        .context("unable to open how-db.toml")?;

    // https://ratatui.rs/faq/#should-i-use-stdout-or-stderr
    // same as `ratatui::restore()` but with stderr instead.
    set_panic_hook();
    enable_raw_mode()?;
    stderr().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(BufWriter::new(stderr())))?;
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

    restore()?;

    if let Some(s) = output {
        println!("{s}");
    }

    Ok(())
}

fn set_panic_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        _ = restore();
        hook(info);
    }));
}

fn restore() -> io::Result<()> {
    disable_raw_mode()?;
    stderr().execute(LeaveAlternateScreen)?;
    Ok(())
}
