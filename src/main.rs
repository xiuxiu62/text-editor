pub mod position;
pub mod text_buffer;
pub mod util;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    style::{self, Color},
    terminal,
};
use std::{
    cell::{RefCell, RefMut},
    collections::BTreeMap,
    io::{self, Stdout},
    path::PathBuf,
    rc::{Rc, Weak},
    time::Duration,
};
use text_buffer::TextBuffer;
use thiserror::Error;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    text::Spans,
    widgets::Paragraph,
    Terminal,
};

macro_rules! btree_map {
    ($($key:expr => $value:expr),*) => {{
        let mut map = BTreeMap::new();
        $(map.insert($key, $value);)*

        map
    }}
}

fn main() -> Result<()> {
    let mut editor = Editor::new()?;
    editor.initialize()?;

    Ok(())
}

type CrossTerminal = Terminal<CrosstermBackend<Stdout>>;

struct Editor {
    active_buffer_id: usize,
    buffers: BTreeMap<usize, TextBuffer>,
    terminal: Option<CrossTerminal>,
}

impl Editor {
    pub fn new() -> text_buffer::Result<Self> {
        Ok(Self {
            active_buffer_id: 1,
            buffers: btree_map![1 => TextBuffer::load(PathBuf::from("./test-data.txt"))?],
            terminal: None,
        })
    }

    pub fn initialize(&mut self) -> Result<()> {
        let mut stdout = io::stdout();
        terminal::enable_raw_mode()?;
        execute!(
            stdout,
            terminal::EnterAlternateScreen,
            terminal::EnableLineWrap,
            event::EnableMouseCapture,
            style::SetBackgroundColor(Color::Black),
            style::SetForegroundColor(Color::White),
            style::SetUnderlineColor(Color::Magenta),
        )?;

        let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
        terminal.clear()?;

        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)])
                .split(frame.size());

            let paragraph = Paragraph::new(
                self.get_active()
                    .unwrap()
                    .lines()
                    .into_iter()
                    .map(Spans::from)
                    .collect::<Vec<Spans>>(),
            );

            frame.render_widget(paragraph, chunks[0]);
        })?;

        let timeout = Duration::from_millis(100);
        loop {
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if let KeyCode::Esc = key.code {
                        break;
                    }
                }
            }
        }

        self.terminal = Some(terminal);

        Ok(())
    }

    fn terminal(&self) -> Result<&CrossTerminal> {
        self.terminal.as_ref().ok_or(Error::Uninitialized)
    }

    fn terminal_mut(&mut self) -> Result<&mut CrossTerminal> {
        self.terminal.as_mut().ok_or(Error::Uninitialized)
    }

    fn get_active(&self) -> Option<&TextBuffer> {
        self.buffers.get(&self.active_buffer_id)
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            active_buffer_id: 1,
            buffers: btree_map![1 => TextBuffer::default()],
            terminal: None,
        }
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        let backend = match self.terminal_mut() {
            Ok(terminal) => terminal.backend_mut(),
            // Terminal was never initialized, so we don't need to run the shutdown hook
            Err(Error::Uninitialized) => return,
            _ => unreachable!(),
        };

        let handle_error = |result| {
            if let Err(err) = result {
                eprintln!("{err}");
            }
        };

        handle_error(execute!(
            backend,
            terminal::LeaveAlternateScreen,
            terminal::DisableLineWrap,
            event::DisableMouseCapture
        ));

        handle_error(terminal::disable_raw_mode());
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    TextBuffer(#[from] text_buffer::Error),
    #[error("Editor unintialized")]
    Uninitialized,
}
