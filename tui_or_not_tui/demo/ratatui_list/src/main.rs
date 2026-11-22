use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{
        Constraint::{Length, Min},
        Layout,
    },
    style::{Style, Stylize},
    widgets::{Block, List, ListState, Paragraph},
    DefaultTerminal, Frame,
};
use std::path::{Path, PathBuf};

// "..", then folders (ending in "/"), then files
fn list_dir(dir: &Path) -> Vec<String> {
    let mut names = vec![];
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if entry.path().is_dir() {
                names.push(name + "/");
            } else {
                names.push(name);
            }
        }
    }
    names.sort_by_key(|name| (!name.ends_with('/'), name.clone()));
    names.insert(0, "..".to_string());
    names
}

// 1. Draw the current state
fn draw(frame: &mut Frame, dir: &Path, state: &mut ListState) {
    let layout = Layout::vertical([Min(0), Length(1)]); // list, help bar
    let [main, help] = layout.areas(frame.area());

    let title = format!(" {} ", dir.display());
    let list = List::new(list_dir(dir))
        .block(Block::bordered().title(title.bold().magenta()))
        .highlight_style(Style::new().reversed())
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, main, state);
    frame.render_widget(
        Paragraph::new(" j/k move, Enter open, Backspace up, q quit").dim(),
        help,
    );
}

// 2. Handle input and update state. Returns false when it's time to quit.
fn handle_events(dir: &mut PathBuf, state: &mut ListState) -> std::io::Result<bool> {
    if let Event::Key(key) = event::read()? {
        if key.kind != KeyEventKind::Press {
            return Ok(true);
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(false),
            KeyCode::Char('j') | KeyCode::Down => state.select_next(),
            KeyCode::Char('k') | KeyCode::Up => state.select_previous(),
            KeyCode::Enter | KeyCode::Char('l') => {
                let name = &list_dir(dir)[state.selected().unwrap_or(0)];
                if name == ".." {
                    dir.pop();
                } else if name.ends_with('/') {
                    dir.push(name.trim_end_matches('/'));
                } else {
                    return Ok(true); // it's a file, nothing to open
                }
                state.select(Some(0));
            }
            KeyCode::Backspace | KeyCode::Char('h') => {
                dir.pop();
                state.select(Some(0));
            }
            _ => {}
        }
    }
    Ok(true)
}

// 3. Loop until done
fn run(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut dir = std::env::current_dir()?; // which folder we're in
    let mut state = ListState::default().with_selected(Some(0)); // which row is selected
    loop {
        terminal.draw(|frame| draw(frame, &dir, &mut state))?; // DRAW
        let keep_going = handle_events(&mut dir, &mut state)?; // WAIT + UPDATE
        if !keep_going {
            return Ok(());
        }
    }
}

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init(); // raw mode + alternate screen
    let result = run(&mut terminal);
    ratatui::restore(); // put the terminal back how we found it
    result
}
