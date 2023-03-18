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
    cell::{Ref, RefCell, RefMut},
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
    widgets::{List, ListItem, Paragraph, StatefulWidget},
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
    terminal: Option<Rc<RefCell<CrossTerminal>>>,
}

enum Message {
    Exit,
}

// enum BufferMode {
//     Normal,
//     Insert,
//     Visual,
// }

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
        self.terminal = Some(Rc::new(RefCell::new(terminal)));

        // self.terminal_mut()?.draw(|frame| {
        //     let chunks = Layout::default()
        //         .direction(Direction::Vertical)
        //         .constraints([Constraint::Percentage(100)])
        //         .split(frame.size());

        //     let paragraph = Paragraph::new(
        //         self.get_active()
        //             .unwrap()
        //             .lines()
        //             .into_iter()
        //             .map(Spans::from)
        //             .collect::<Vec<Spans>>(),
        //     );

        //     frame.render_widget(paragraph, chunks[0]);
        // })?;

        loop {
            if let Some(message) = self.update()? {
                match message {
                    Message::Exit => break,
                }
            }

            self.render()?;

            // TODO: implement actual FPS
            std::thread::sleep(Duration::from_nanos(16_667));
        }

        Ok(())
    }

    fn update(&mut self) -> Result<Option<Message>> {
        let timeout = Duration::from_millis(100);
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => return Ok(Some(Message::Exit)),
                    KeyCode::Char(char) => {
                        let text_buffer = self.get_active_mut().unwrap();

                        text_buffer.insert_char(text_buffer.cursor, char)?;
                        text_buffer.cursor.set_x(text_buffer.cursor.x() + 1);
                    }
                    _ => {}
                }
            }
        }

        Ok(None)
    }

    fn render(&self) -> Result<()> {
        self.terminal_mut()?.draw(|frame| {
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

        Ok(())
    }

    fn terminal(&self) -> Result<Ref<CrossTerminal>> {
        self.terminal
            .as_ref()
            .ok_or(Error::Uninitialized)
            .map(|terminal| terminal.borrow())
    }

    fn terminal_mut(&self) -> Result<RefMut<CrossTerminal>> {
        self.terminal
            .as_ref()
            .ok_or(Error::Uninitialized)
            .map(|terminal| terminal.borrow_mut())
    }

    fn get_active(&self) -> Option<&TextBuffer> {
        self.buffers.get(&self.active_buffer_id)
    }

    fn get_active_mut(&mut self) -> Option<&mut TextBuffer> {
        self.buffers.get_mut(&self.active_buffer_id)
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
        // If Err, the terminal wasn't initialized and we don't need to run the shutdown hook
        if let Ok(mut terminal) = self.terminal_mut() {
            let handle_error = |result| {
                if let Err(err) = result {
                    eprintln!("{err}");
                }
            };

            handle_error(execute!(
                terminal.backend_mut(),
                terminal::LeaveAlternateScreen,
                terminal::DisableLineWrap,
                event::DisableMouseCapture
            ));

            handle_error(terminal::disable_raw_mode());
        };
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
