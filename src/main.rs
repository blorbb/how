mod db;
mod rank;
mod ui;

use std::fs;

use color_eyre::{
    eyre::{Context, ContextCompat},
    Result,
};
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
    App::new(data).run(&mut terminal)?;

    ratatui::restore();
    Ok(())
}
