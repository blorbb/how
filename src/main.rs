mod db;
mod rank;
mod ui;
mod utils;
mod widgets;

use std::{
    fs,
    io::{self, stderr, BufWriter, Write},
    process::{self, Command},
};

use clap::Parser;
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

#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// Immediately executes the command instead of printing to stdout.
    #[arg(long)]
    execute: bool,
    /// An initial query to insert. Can be quoted or unquoted,
    /// in which case, each argument will be separated by a space.
    ///
    /// WARNING: if inserting unquoted, any word that starts with a dash
    /// may be interpreted as a flag instead. Quoted strings that start
    /// with a dash may also be interpreted as a flag.
    ///
    /// To avoid accidentally setting flags, insert text after a `--`.
    ///
    /// For example: `how --execute -- initial -h query`.
    /// This sets the `--execute` flag, and has an initial query of
    /// "initial -h query".
    query: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

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
    let mut app = App::new(data, args.query.join(" "));
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
        if args.execute {
            let output = if cfg!(target_os = "windows") {
                Command::new("cmd").arg("/C").arg(&s).output()?
            } else {
                Command::new("sh").arg("-c").arg(&s).output()?
            };

            // show output of that command
            _ = io::stdout().write_all(&output.stdout);
            _ = io::stderr().write_all(&output.stderr);
            if let Some(code) = output.status.code() {
                process::exit(code);
            }
        } else {
            println!("{s}");
        }
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
