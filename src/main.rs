mod db;

use std::fs;

use color_eyre::{
    eyre::{Context, ContextCompat},
    Result,
};
use db::{Data, Entry};

fn main() -> Result<()> {
    let dir = dirs::data_dir().context("unable to find data directory")?;

    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(dir.join("how-db.toml"))
        .context("unable to open how-db.toml")?;

    let mut entries = Data::load_from(file)?;

    entries.add(Entry::new("Test title", "Description thing", "answer is this"))?;

    Ok(())
}
