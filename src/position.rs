use std::{fmt::Display, ops::Range};

/// Non-inclusive offset span
pub type Span = Range<usize>;

/// Item wrapped in a non-inclusive offset span
pub type Spanned<I> = (I, Span);

/// 2-Dimensional coordiante
#[derive(Debug, Default, Clone, Copy)]
pub struct Point(usize, usize);

impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

impl Point {
    #[inline]
    pub const fn new(x: usize, y: usize) -> Self {
        Self(x, y)
    }

    #[inline]
    pub const fn x(&self) -> usize {
        self.0
    }

    #[inline]
    pub const fn y(&self) -> usize {
        self.1
    }

    #[inline]
    pub fn set_x(&mut self, to: usize) {
        self.0 = to;
    }

    #[inline]
    pub fn set_y(&mut self, to: usize) {
        self.1 = to;
    }

    /// Gives a copy of the inner tuple
    #[inline]
    pub const fn into_inner(&self) -> (usize, usize) {
        (self.0, self.1)
    }
}
