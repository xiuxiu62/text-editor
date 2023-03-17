pub mod position;
pub mod text_buffer;
pub mod util;

use std::path::PathBuf;
use text_buffer::TextBuffer;

fn main() -> text_buffer::Result<()> {
    let mut buffer = TextBuffer::load(PathBuf::from("./test-data.txt"))?;
    buffer.insert_line(3, "meow");

    buffer.save()
}

// struct LineDescriptor {
//     size: usize,
//     modified: bool,
// }
