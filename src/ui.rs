use std::{cell::RefCell, cmp, num::Saturating, rc::Rc};

use color_eyre::Result;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    widgets::{StatefulWidget, Widget},
    DefaultTerminal,
};
use ratatui_macros::{horizontal, line, vertical};
use tui_textarea::{Input, Key};
use tui_widget_list::{ListBuilder, ListState, ListView};

use crate::{
    db::{Data, Entry},
    rank,
    utils::{Action, Wrapping},
    widgets::{ConfirmDialog, TextArea},
};

pub struct App {
    data: Rc<RefCell<Data>>,
    query: TextArea,
    matches: Vec<(usize, f32)>,
    list_index: Saturating<usize>,
    entry_editor: Option<EntryEditor>,
    dialog: Option<ConfirmDialog<Self>>,
}

impl App {
    pub fn new(data: Data) -> Self {
        Self {
            matches: rank::rank("", data.entries()),
            data: Rc::new(RefCell::new(data)),
            query: TextArea::new_focused("", "Search").set_single_line(),
            list_index: Saturating(0),
            entry_editor: None,
            dialog: None,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.draw(terminal)?;

        loop {
            self.draw(terminal)?;
            if let Event::Key(k) = event::read()? {
                let KeyEventKind::Press = k.kind else {
                    continue;
                };
                let k = Input::from(k);

                if let Some(dialog) = self.dialog.take() {
                    match dialog.read(k) {
                        Some(true) => dialog.execute(self)?,
                        Some(false) => {}
                        None => self.dialog = Some(dialog),
                    };
                    continue;
                } else if let Some(entry_editor) = &mut self.entry_editor {
                    match entry_editor.read(k) {
                        Some(Action::Exit) => self.close_entry_editor(),
                        Some(Action::AddEntry(entry)) => {
                            self.dialog = Some(ConfirmDialog::new(
                                "Are you sure you want to create a new log?",
                                |app: &mut App| {
                                    app.data.borrow_mut().add(entry)?;
                                    app.close_entry_editor();
                                    Ok(())
                                },
                            ));
                        }
                        None => {}
                    }
                    continue;
                }

                // main screen
                match k {
                    Input { key: Key::Esc, .. } => break,
                    Input {
                        key: Key::Char('a'),
                        ctrl: true,
                        ..
                    } => self.add_new(),
                    Input { key: Key::Down, .. } => self.next_item(),
                    Input { key: Key::Up, .. } => self.prev_item(),
                    _ => self.register_input(k),
                }
            }
        }

        Ok(())
    }

    fn next_item(&mut self) {
        self.list_index = Saturating((self.list_index.0 + 1).min(self.matches.len() - 1))
    }

    fn prev_item(&mut self) {
        self.list_index -= 1
    }

    fn register_input(&mut self, ev: Input) {
        self.query.input(ev);
        self.refresh_list();
    }

    fn refresh_list(&mut self) {
        let borrow = self.data.borrow();
        self.matches = rank::rank(self.query_text(), borrow.entries());
        self.list_index = Saturating(0);
    }

    fn draw(&self, terminal: &mut DefaultTerminal) -> Result<()> {
        terminal.draw(|frame| {
            frame.render_widget(self, frame.area());
        })?;
        Ok(())
    }

    fn query_text(&self) -> &str {
        &self.query.lines()[0].trim()
    }

    fn close_entry_editor(&mut self) {
        self.entry_editor = None;
        self.query.focus();
        self.refresh_list();
    }

    fn add_new(&mut self) {
        self.entry_editor = Some(EntryEditor::new(self.query_text(), "", ""));
        self.query.blur();
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let (query_area, list_area, pane_area) = {
            let hor = horizontal![==1/2; 2].split(area);
            let vert = vertical![==3, *=1].split(hor[0]);
            (vert[0], vert[1], hor[1])
        };
        let data = Rc::clone(&self.data);
        let matches = self.matches.clone();

        let builder = ListBuilder::new(move |cx| {
            let item = data.borrow().entries()[matches[cx.index].0].clone();
            let title = line![
                item.title().to_string(),
                format!(" ({:.4})", matches[cx.index].1)
            ];
            let title = if cx.is_selected {
                title.on_dark_gray().bold().yellow()
            } else {
                title
            };

            (title, 1)
        });
        let list = ListView::new(builder, self.matches.len());

        let mut list_state = ListState::default();
        list_state.select(Some(self.list_index.0));

        self.query.render(query_area, buf);
        list.render(list_area, buf, &mut list_state);

        if let Some(entry_editor) = &self.entry_editor {
            entry_editor.render(pane_area, buf);
        } else {
            let binding = self.data.borrow();
            let selected = &binding.entries()[self.matches[self.list_index.0].0];
            selected.render(pane_area, buf);
        }

        if let Some(dialog) = &self.dialog {
            dialog.render(area, buf);
        }
    }
}

struct EntryEditor {
    title: TextArea,
    code: TextArea,
    description: TextArea,
    focus: Wrapping<3>,
}

impl EntryEditor {
    pub fn new(
        title: impl Into<String>,
        code: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            title: TextArea::new_focused(title, "Title")
                .set_single_line()
                .set_validator("Title cannot be empty", |s| !s.is_empty()),
            code: TextArea::new_blurred(code, "Code"),
            description: TextArea::new_blurred(description, "Description"),
            focus: Wrapping::default(),
        }
    }

    pub fn read(&mut self, input: Input) -> Option<Action> {
        match input {
            Input {
                key: Key::Tab,
                shift: false,
                ..
            } => self.focus_next(),
            Input {
                // shift-tab is null for some reason??
                key: Key::Null,
                shift: true,
                ..
            } => self.focus_prev(),
            Input { key: Key::Esc, .. } => return Some(Action::Exit),
            Input {
                key: Key::Char('s'),
                ctrl: true,
                ..
            } if self.is_valid() => {
                return Some(Action::AddEntry(Entry::new(
                    self.title.text(),
                    self.code.text(),
                    self.description.text(),
                )))
            }
            _ => self.current_area().input(input),
        }

        None
    }

    fn focus_next(&mut self) {
        self.current_area().blur();
        self.focus.next();
        self.current_area().focus();
    }

    fn focus_prev(&mut self) {
        self.current_area().blur();
        self.focus.prev();
        self.current_area().focus();
    }

    fn current_area(&mut self) -> &mut TextArea {
        match self.focus.get() {
            0 => &mut self.title,
            1 => &mut self.code,
            2 => &mut self.description,
            _ => unreachable!(),
        }
    }

    fn is_valid(&self) -> bool {
        self.title.is_valid() && self.code.is_valid() && self.description.is_valid()
    }
}

impl Widget for &EntryEditor {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        // +2 for borders
        let title_height = cmp::max(1, self.title.lines().len() as u16) + 2;
        let code_height = cmp::max(1, self.code.lines().len() as u16) + 2;

        let layout = vertical![==title_height, ==code_height, *=1].split(area);
        self.title.render(layout[0], buf);
        self.code.render(layout[1], buf);
        self.description.render(layout[2], buf);
    }
}
