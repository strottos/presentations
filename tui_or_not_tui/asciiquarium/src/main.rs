mod art;
mod entity;
mod fish;
mod world;

use std::{
    io,
    time::{Duration, Instant},
};

use ratatui::{
    DefaultTerminal,
    crossterm::{
        event::{
            self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
            KeyModifiers, MouseButton, MouseEventKind,
        },
        execute,
    },
};

use world::World;

/// ~20 frames per second.
const TICK: Duration = Duration::from_millis(50);

fn main() -> io::Result<()> {
    // Make sure a panic doesn't leave the terminal reporting mouse events.
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = execute!(io::stdout(), DisableMouseCapture);
        prev_hook(info);
    }));

    let mut terminal = ratatui::init();
    execute!(io::stdout(), EnableMouseCapture)?;
    let result = run(&mut terminal);
    let _ = execute!(io::stdout(), DisableMouseCapture);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let size = terminal.size()?;
    let mut world = World::new(size.width, size.height);
    // Where the mouse last was, so the keyboard can feed "from the hand".
    let mut cursor_col: Option<i32> = None;
    let mut last = Instant::now();

    loop {
        terminal.draw(|frame| frame.render_widget(&world, frame.area()))?;

        // Wait out the rest of the tick, handling input as it arrives.
        let next = last + TICK;
        while let Some(remaining) = next.checked_duration_since(Instant::now()) {
            if !event::poll(remaining)? {
                break;
            }
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(());
                    }
                    KeyCode::Char(' ') | KeyCode::Char('f') => world.feed(cursor_col),
                    KeyCode::Char('t') => world.tap_random(),
                    KeyCode::Char('s') => world.add_special(),
                    KeyCode::Char('a') => world.add_extra_fish(),
                    KeyCode::Char('p') => world.paused = !world.paused,
                    KeyCode::Char('h') | KeyCode::Char('?') => world.show_help = !world.show_help,
                    KeyCode::Char('r') => {
                        let size = terminal.size()?;
                        world.resize(size.width, size.height);
                    }
                    _ => {}
                },
                Event::Mouse(m) => {
                    cursor_col = Some(m.column as i32);
                    match m.kind {
                        MouseEventKind::Down(MouseButton::Left) => {
                            world.drop_food(m.column as i32, m.row as i32, 4);
                        }
                        MouseEventKind::Drag(MouseButton::Left) => {
                            world.drop_food(m.column as i32, m.row as i32, 1);
                        }
                        MouseEventKind::Down(MouseButton::Right) => {
                            world.tap(m.column as i32, m.row as i32);
                        }
                        _ => {}
                    }
                }
                Event::Resize(w, h) => world.resize(w, h),
                _ => {}
            }
        }

        let dt = last.elapsed().as_secs_f32().min(0.1);
        last = Instant::now();
        world.update(dt);
    }
}
