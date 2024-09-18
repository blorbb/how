mod db;
mod rank;

use std::fs;

use color_eyre::{
    eyre::{Context, ContextCompat},
    Result,
};
use db::{Data, Entry};
use rank::rank;

fn main() -> Result<()> {
    let dir = dirs::data_dir().context("unable to find data directory")?;

    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true) // remove when no longer testing!
        .open(dir.join("how-db.toml"))
        .context("unable to open how-db.toml")?;

    let mut data = Data::load_from(file)?;

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

    let query = "a";
    let matches = rank(query, data.entries());

    println!("{matches:#?}");

    Ok(())
}
