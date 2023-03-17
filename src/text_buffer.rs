mod error;
#[cfg(test)]
mod test;

pub use error::{Error, Result};

use crate::{
    position::{Point, Span},
    util,
};
use std::{fs, path::PathBuf};

/// A text buffer
#[derive(Debug, Default)]
pub struct TextBuffer {
    /// Actual data
    data: String,
    /// Line descriptors
    line_sizes: Vec<usize>,
    /// File backing
    pub path: Option<PathBuf>,
    /// Cursor position
    pub cursor: Point,
}

impl TextBuffer {
    /// Load buffer from a file
    pub fn load(path: PathBuf) -> Result<Self> {
        let data = std::fs::read_to_string(&path)?;
        let mut buffer = Self::from(data);
        buffer.path = Some(path);

        Ok(buffer)
    }

    /// Save buffer to it's file
    pub fn save(&self) -> Result<()> {
        let Some(path) = &self.path else {
        return Err(Error::FileNotSet);
    };

        let newline = util::newline();
        let data = self
            .line_sizes
            .iter()
            .fold(
                ("".to_owned(), 0),
                |(mut data, mut current_offset), line_size| {
                    if *line_size > 0 {
                        let line: String = self
                            .data
                            .chars()
                            .skip(current_offset)
                            .take(*line_size)
                            .collect();

                        data.push_str(&line);
                        current_offset += line_size;
                    }

                    data.push_str(newline);

                    (data, current_offset)
                },
            )
            .0;

        fs::write(path, data)?;

        Ok(())
    }

    /// Inserts a line, appending necessary empty lines if the index is out of bounds
    // UNWRAP: we've already ensured the index is in range
    pub fn insert_line(&mut self, row: usize, value: &str) {
        // append
        if row >= self.line_sizes.len() {
            // push empty line descriptors up to our new line
            (self.line_sizes.len()..row).for_each(|_| self.line_sizes.push(0));
            // push the new line descriptor
            self.line_sizes.push(value.len());
            // push the  data
            self.data.push_str(value);

            return;
        }

        // insert
        let starting_offset = (0..row)
            .map(|i| self.line_sizes.get(i))
            .sum::<Option<usize>>()
            .unwrap();

        // insert the line descriptor
        self.line_sizes.insert(row, value.len());
        // insert the data
        self.data.insert_str(starting_offset, value);
    }

    /// Removes a line, returing the removed data if the index is in bounds, otherwise returns none
    // UNWRAP: we've already ensured the index is in range
    pub fn remove_line(&mut self, row: usize) -> Option<String> {
        // index out of bounds
        if row >= self.line_sizes.len() {
            return None;
        }

        let start = (0..row)
            .map(|i| self.line_sizes.get(i))
            .sum::<Option<usize>>()
            .unwrap();
        let end = start + self.line_sizes.remove(row);

        Some(
            (start..end)
                .into_iter()
                .map(|_| self.data.remove(start))
                .collect(),
        )
    }

    /// Inserts a character to a line
    pub fn insert_char(&mut self, at: Point, char: char) -> Result<()> {
        let (offset, line_size) = self.offset(at)?;
        *line_size += 1;

        self.data.insert(offset, char);

        Ok(())
    }

    /// Removes a character, returing the removed value
    pub fn remove_char(&mut self, at: Point) -> Result<char> {
        let (offset, line_size) = self.offset(at)?;
        *line_size -= 1;

        Ok(self.data.remove(offset))
    }

    /// Replaces a charater, returing the old value
    // UNWRAP: we've ensured the index is valid in Self::offset_span
    pub fn replace_char(&mut self, at: Point, char: char) -> Result<char> {
        self.replace_str(at, char.to_string().as_str())
            .map(|old| old.chars().nth(0).unwrap())
    }

    /// Adds to a line
    pub fn insert_str(&mut self, at: Point, value: &str) -> Result<()> {
        let (span, line_size) = self.offset_span(at, value.len())?;
        *line_size += value.len();

        self.data.insert_str(span.start, value);

        Ok(())
    }

    /// Removes part of a line, returning the removed value
    pub fn remove_str(&mut self, at: Point, size: usize) -> Result<String> {
        let (span, line_size) = self.offset_span(at, size)?;
        *line_size -= size;

        let starting_offset = span.start;
        Ok(span
            .into_iter()
            .map(|_| self.data.remove(starting_offset))
            .collect())
    }

    /// Replaces part of a line, returning the old value
    pub fn replace_str(&mut self, at: Point, value: &str) -> Result<String> {
        let (span, _) = self.offset_span(at, value.len())?;
        let old_value = self
            .data
            .chars()
            .skip(span.start)
            .take(span.end - span.start)
            .collect();

        self.data.replace_range(span, value);

        Ok(old_value)
    }

    /// Gives the character offset from the inner data, along with a mutable reference to the corresponding line size
    fn offset(&mut self, point: Point) -> Result<(usize, &mut usize)> {
        let (column, row) = point.into_inner();
        let starting_offset: usize = (0..row)
            .map(|i| self.line_sizes.get(i))
            .sum::<Option<usize>>()
            .ok_or(Error::OutOfBounds(point))?;
        let line_size = self
            .line_sizes
            .get_mut(row)
            .ok_or(Error::OutOfBounds(point))?;
        if column >= *line_size {
            return Err(Error::OutOfBounds(point));
        }

        Ok((starting_offset + column, line_size))
    }

    /// Gives the character offset from the inner data, along with a mutable reference to the corresponding line size
    fn offset_span(&mut self, point: Point, size: usize) -> Result<(Span, &mut usize)> {
        let (column, row) = point.into_inner();
        let starting_offset: usize = (0..row)
            .map(|i| self.line_sizes.get(i))
            .sum::<Option<usize>>()
            .ok_or(Error::OutOfBounds(point))?;
        let line_size = self
            .line_sizes
            .get_mut(row)
            .ok_or(Error::OutOfBounds(point))?;
        if column + size + 1 > *line_size {
            return Err(Error::OutOfBounds(point));
        }

        Ok((
            Span {
                start: starting_offset + column,
                end: starting_offset + column + size + 1,
            },
            line_size,
        ))
    }

    // fn in_bounds(&self, point: Point) -> bool {
    //     match self.lines.get(point.y()).map(|line| point.x() <= line.end) {
    //         Some(true) => true,
    //         _ => false,
    //     }
    // }
}

impl<S: Into<String>> From<S> for TextBuffer {
    fn from(data: S) -> Self {
        let (data, line_sizes) =
            data.into()
                .lines()
                .fold(("".to_owned(), vec![]), |(data, mut line_sizes), line| {
                    line_sizes.push(line.len());

                    (format!("{data}{line}"), line_sizes)
                });

        Self {
            data,
            line_sizes,
            path: None,
            cursor: Point::default(),
        }
    }
}
