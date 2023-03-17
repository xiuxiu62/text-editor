pub mod position;
pub mod text_buffer;
pub mod util;

use crossterm::{
    event, execute,
    style::{self, Color},
    terminal,
};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    io::{self, Stdout},
    path::PathBuf,
    rc::{Rc, Weak},
};
use text_buffer::TextBuffer;
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

fn main() -> text_buffer::Result<()> {
    // let mut buffer = TextBuffer::load(PathBuf::from("./test-data.txt"))?;
    let mut editor = Editor::new()?;
    editor.initialize()?;

    Ok(())

    // buffer.insert_line(3, "meow");
    // Editor::default()

    // buffer.save()
}

type WeakTermRef = Weak<RefCell<Terminal<CrosstermBackend<Stdout>>>>;

struct Editor {
    active_buffer_id: usize,
    buffers: BTreeMap<usize, TextBuffer>,
    terminal: WeakTermRef,
}

impl Editor {
    pub fn new() -> text_buffer::Result<Self> {
        let mut buffer = TextBuffer::load(PathBuf::from("./test-data.txt"))?;

        Ok(Self {
            active_buffer_id: 1,
            buffers: btree_map![1 => buffer],
            terminal: Weak::new(),
        })
    }

    pub fn initialize(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();
        execute!(
            stdout,
            terminal::EnterAlternateScreen,
            terminal::EnableLineWrap,
            event::EnableMouseCapture,
            style::SetBackgroundColor(Color::Black),
            style::SetForegroundColor(Color::White),
            style::SetForegroundColor(Color::Magenta),
        )?;
        terminal::enable_raw_mode()?;

        let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
        terminal.clear()?;

        // self.terminal = Weak::from(RefCell::new(terminal));

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

        Ok(())
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
            terminal: Weak::new(),
        }
    }
}
