#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Position {
    pub(crate) offset: usize,
    pub(crate) line: usize,
    pub(crate) col: usize,
}

mod stream;