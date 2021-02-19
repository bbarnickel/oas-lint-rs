use std::str::CharIndices;

use super::Position;

pub(crate) struct Stream<'a> {
    char_indices: CharIndices<'a>,
    col: usize,
    offset: usize,
    line: usize,
    peek: Option<(usize, char)>,
    had_linebreak: bool,
}

impl<'a> Stream<'a> {
    pub(crate) fn new(data: &'a str) -> Self {
        Self {
            char_indices: data.char_indices(),
            col: 0,
            offset: 0,
            line: 1,
            peek: None,
            had_linebreak: false,
        }
    }

    pub(crate) fn get_position(&self) -> Position {
        Position {
            offset: self.offset,
            line: self.line,
            col: self.col,
        }
    }

    pub(crate) fn next(&mut self) -> Option<char> {
        match self.next_internal() {
            Some((i, '\r')) => {
                self.update_pos(i);

                match self.next_internal() {
                    Some((_, '\n')) => {
                        self.had_linebreak = true;
                        Some('\n')
                    }
                    Some((j, cc)) => {
                        self.peek = Some((j, cc));
                        self.had_linebreak = true;
                        Some('\n')
                    }
                    None => {
                        self.had_linebreak = true;
                        Some('\n')
                    }
                }
            }
            Some((i, '\n')) => {
                self.update_pos(i);
                self.had_linebreak = true;
                Some('\n')
            }
            Some((i, c)) => {
                self.update_pos(i);
                Some(c)
            }
            None => None,
        }
    }

    fn next_internal(&mut self) -> Option<(usize, char)> {
        match self.peek {
            None => self.char_indices.next(),
            _ => self.peek.take(),
        }
    }

    fn update_pos(&mut self, offset: usize) {
        self.offset = offset;
        if self.had_linebreak {
            self.line += 1;
            self.col = 1;
            self.had_linebreak = false;
        } else {
            self.col += 1;
        }
    }
}

#[cfg(test)]
mod test_stream {
    use super::*;

    #[test]
    fn test_empty() {
        let mut stream = Stream::new("");
        assert_end(&mut stream);
    }

    #[test]
    fn test_single_word() {
        let mut stream = Stream::new("AB");
        assert_char(&mut stream, 'A', 0, 1, 1);
        assert_char(&mut stream, 'B', 1, 1, 2);
        assert_end(&mut stream);
    }

    #[test]
    fn test_single_word_and_n() {
        let mut stream = Stream::new("AB\n");
        assert_char(&mut stream, 'A', 0, 1, 1);
        assert_char(&mut stream, 'B', 1, 1, 2);
        assert_char(&mut stream, '\n', 2, 1, 3);
        assert_end(&mut stream);
    }

    #[test]
    fn test_single_word_and_r() {
        let mut stream = Stream::new("AB\r");
        assert_char(&mut stream, 'A', 0, 1, 1);
        assert_char(&mut stream, 'B', 1, 1, 2);
        assert_char(&mut stream, '\n', 2, 1, 3);
        assert_end(&mut stream);
    }

    #[test]
    fn test_single_word_and_rn() {
        let mut stream = Stream::new("AB\r\n");
        assert_char(&mut stream, 'A', 0, 1, 1);
        assert_char(&mut stream, 'B', 1, 1, 2);
        assert_char(&mut stream, '\n', 2, 1, 3);
        assert_end(&mut stream);
    }

    #[test]
    fn test_word_n_word() {
        let mut stream = Stream::new("AB\nCD");
        assert_char(&mut stream, 'A', 0, 1, 1);
        assert_char(&mut stream, 'B', 1, 1, 2);
        assert_char(&mut stream, '\n', 2, 1, 3);
        assert_char(&mut stream, 'C', 3, 2, 1);
        assert_char(&mut stream, 'D', 4, 2, 2);
        assert_end(&mut stream);
    }

    #[test]
    fn test_word_r_word() {
        let mut stream = Stream::new("AB\rCD");
        assert_char(&mut stream, 'A', 0, 1, 1);
        assert_char(&mut stream, 'B', 1, 1, 2);
        assert_char(&mut stream, '\n', 2, 1, 3);
        assert_char(&mut stream, 'C', 3, 2, 1);
        assert_char(&mut stream, 'D', 4, 2, 2);
        assert_end(&mut stream);
    }

    #[test]
    fn test_word_rn_word() {
        let mut stream = Stream::new("AB\r\nCD");
        assert_char(&mut stream, 'A', 0, 1, 1);
        assert_char(&mut stream, 'B', 1, 1, 2);
        assert_char(&mut stream, '\n', 2, 1, 3);
        assert_char(&mut stream, 'C', 4, 2, 1);
        assert_char(&mut stream, 'D', 5, 2, 2);
        assert_end(&mut stream);
    }

    #[test]
    fn test_emoji_end() {
        let mut stream = Stream::new("Hi 😊");
        assert_char(&mut stream, 'H', 0, 1, 1);
        assert_char(&mut stream, 'i', 1, 1, 2);
        assert_char(&mut stream, ' ', 2, 1, 3);
        assert_char(&mut stream, '😊', 3, 1, 4);
        assert_end(&mut stream);
    }

    #[test]
    fn test_emoji_start() {
        // emoji is 4 bytes long
        let mut stream = Stream::new("😊 Bye!");
        assert_char(&mut stream, '😊', 0, 1, 1);
        assert_char(&mut stream, ' ', 4, 1, 2);
        assert_char(&mut stream, 'B', 5, 1, 3);
        assert_char(&mut stream, 'y', 6, 1, 4);
        assert_char(&mut stream, 'e', 7, 1, 5);
        assert_char(&mut stream, '!', 8, 1, 6);
        assert_end(&mut stream);
    }

    #[test]
    fn test_emoji_n_emoji() {
        let mut stream = Stream::new("😊\n😊");
        assert_char(&mut stream, '😊', 0, 1, 1);
        assert_char(&mut stream, '\n', 4, 1, 2);
        assert_char(&mut stream, '😊', 5, 2, 1);
        assert_end(&mut stream);
    }

    #[test]
    fn test_emoji_r_emoji() {
        let mut stream = Stream::new("😊\r😊");
        assert_char(&mut stream, '😊', 0, 1, 1);
        assert_char(&mut stream, '\n', 4, 1, 2);
        assert_char(&mut stream, '😊', 5, 2, 1);
        assert_end(&mut stream);
    }

    #[test]
    fn test_emoji_rn_emoji() {
        let mut stream = Stream::new("😊\r\n😊");
        assert_char(&mut stream, '😊', 0, 1, 1);
        assert_char(&mut stream, '\n', 4, 1, 2);
        assert_char(&mut stream, '😊', 6, 2, 1);
        assert_end(&mut stream);
    }

    fn assert_char(stream: &mut Stream, value: char, offset: usize, line: usize, col: usize) {
        if let Some(c) = stream.next() {
            assert_eq!(c, value);
            assert_eq!(offset, stream.offset, "Offset");
            assert_eq!(line, stream.line, "Line");
            assert_eq!(col, stream.col, "Column");
        } else {
            panic!("Char expected!");
        }
    }

    fn assert_end(stream: &mut Stream) {
        let val = stream.next();
        assert!(val.is_none());
    }
}
