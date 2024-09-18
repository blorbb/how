use std::{
    fs::File,
    io::{Read, Seek, Write},
};

use color_eyre::eyre::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    title: String,
    description: String,
    answer: String,
    used: u32,
}

impl Entry {
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
        answer: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            answer: answer.into(),
            used: 0,
        }
    }
}

// needed for derive to get the correct key
#[derive(Debug, Serialize, Deserialize)]
struct Entries {
    entries: Vec<Entry>,
}

impl Entries {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct Data {
    entries: Entries,
    file: File,
}

impl Data {
    pub fn load_from(mut file: File) -> Result<Self> {
        let mut str = String::new();
        file.read_to_string(&mut str).context("corrupted file")?;
        let entries = if str.trim().is_empty() {
            Data {
                entries: Entries::new(),
                file,
            }
        } else {
            Data {
                entries: toml_edit::de::from_str(&str)?,
                file,
            }
        };

        Ok(entries)
    }

    pub fn add(&mut self, entry: Entry) -> Result<()> {
        self.entries.entries.push(entry);
        let doc = toml_edit::ser::to_string_pretty(&self.entries)?;
        self.file.set_len(0)?;
        self.file.rewind()?;
        self.file.write_all(doc.as_bytes())?;
        Ok(())
    }
}
