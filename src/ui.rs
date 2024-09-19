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
    db::Data,
    rank,
    utils::{Action, TextArea, Wrapping},
};

pub struct App {
    data: Rc<RefCell<Data>>,
    query: TextArea,
    matches: Vec<(usize, f32)>,
    list_index: Saturating<usize>,
    entry_editor: Option<EntryEditor>,
}

impl App {
    pub fn new(data: Data) -> Self {
        Self {
            matches: rank::rank("", data.entries()),
            data: Rc::new(RefCell::new(data)),
            query: TextArea::new_focused("", "Search"),
            list_index: Saturating(0),
            entry_editor: None,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.draw(terminal)?;

        loop {
            if let Event::Key(k) = event::read()? {
                let KeyEventKind::Press = k.kind else {
                    continue;
                };
                let k = Input::from(k);

                if let Some(entry_editor) = &mut self.entry_editor {
                    let action = entry_editor.read(k);
                    match action {
                        Some(Action::Exit) => {
                            self.entry_editor = None;
                            self.query.focus();
                        }
                        None => {}
                    }
                    self.draw(terminal)?;
                    continue;
                }

                // main screen
                match k {
                    Input {
                        key: Key::Char('q'),
                        ctrl: true,
                        ..
                    } => break,
                    Input {
                        key: Key::Char('a'),
                        ctrl: true,
                        ..
                    } => self.add_new(),
                    Input { key: Key::Down, .. } => self.next_item(),
                    Input { key: Key::Up, .. } => self.prev_item(),
                    _ => self.register_input(k),
                }

                self.draw(terminal)?;
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

    fn add_new(&mut self) {
        self.entry_editor = Some(EntryEditor::new(self.query_text(), "", ""));
        self.query.blur();
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
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
            let item = Rc::clone(&data.borrow().entries()[matches[cx.index].0]);
            let title = line![
                item.title().to_string(),
                format!(" ({:.4})", matches[cx.index].1)
            ];
            let title = if cx.is_selected {
                title.on_gray()
            } else {
                title
            };

            (title, 1)
        });
        let list = ListView::new(builder, self.matches.len().min(50));

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
            title: TextArea::new_focused(title, "Title"),
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
            Input {
                key: Key::Char('q'),
                ctrl: true,
                ..
            } => return Some(Action::Exit),
            _ => _ = self.current_area().input(input),
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
