use std::{
    cmp,
    fs::File,
    io::{Read, Seek, Write},
    iter,
};

use color_eyre::eyre::{Context, Result};
use ratatui::{
    style::Stylize,
    widgets::{Block, Paragraph, Widget},
};
use ratatui_macros::vertical;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Entry {
    title: String,
    answer: String,
    description: String,
    used: u32,
}

impl Entry {
    pub fn new(
        title: impl Into<String>,
        answer: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            answer: answer.into(),
            used: 0,
        }
    }

    /// Converts the entry into one string that should be searched
    /// for fuzzy finding.
    pub fn to_haystack(&self) -> String {
        self.title
            .chars()
            .chain(iter::once('\n'))
            .chain(self.description.chars())
            .chain(iter::once('\n'))
            .chain(self.answer.chars())
            .collect()
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn answer(&self) -> &str {
        &self.answer
    }
}

impl Widget for &Entry {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block = Block::bordered();
        // +2 for borders
        let code_height = cmp::max(1, self.answer().lines().count() as u16) + 2;
        let layout = vertical![==1, ==1, ==code_height, ==1, *=1].split(block.inner(area));

        let title = self.title().bold();
        let code_block = Paragraph::new(self.answer()).block(Block::bordered().title("Command"));
        block.render(area, buf);
        title.render(layout[0], buf);
        code_block.render(layout[2], buf);
        self.description().render(layout[4], buf);
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

    pub fn entries(&self) -> &[Entry] {
        &self.entries.entries
    }
}
