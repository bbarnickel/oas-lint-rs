use super::{Position, stream::Stream};

#[derive(Debug, PartialEq)]
pub(crate) struct Token<'a> {
    pos: Position,
    content: TokenType<'a>,
}

impl<'a> Token<'a> {
    fn new(pos: Position, content: TokenType<'a>) -> Self {
        Self { pos, content }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum TokenType<'a> {
    Spaces(usize),
    QuestionMark,
    Dash,
    Newline,
    Colon,
    LeftBraket,
    RightBraket,
    LeftSqBraket,
    RightSqBraket,
    DoubleQuote,
    SingleQuote,
    // BlockLiteralSign,
    // BlockJoinSigns,
    String(&'a str),
}

pub(crate) struct Scanner<'a> {
    data: &'a str,
    stream: Stream<'a>,
    peeked_char: Option<char>,
    in_double_quote: bool,
    in_single_quote: bool,
}

impl<'a> Scanner<'a> {
    pub(crate) fn new(data: &'a str) -> Self {
        let mut result = Self {
            data,
            stream: Stream::new(data),
            peeked_char: None,
            in_double_quote: false,
            in_single_quote: false,
        };

        result
    }

    pub(crate) fn next(&mut self) -> Option<Token> {
        let (c, pos) = match self.peeked_char {
            Some(cc) => (self.peeked_char.take(), self.stream.get_position()),
            None => {
                let cc = self.stream.next();
                (cc, self.stream.get_position())
            }
        };

        let in_quote = self.in_double_quote || self.in_single_quote;

        match c {
            Some(cc) => match cc {
                ' ' if pos.col == 1 => self.create_indent(pos),
                '"' if !in_quote => {
                    self.in_double_quote = true;
                    self.create_token(pos, TokenType::DoubleQuote)
                }
                '"' if self.in_double_quote => {
                    self.in_double_quote = false;
                    self.create_token(pos, TokenType::DoubleQuote)
                }
                '\'' if !in_quote => {
                    self.in_single_quote = true;
                    self.create_token(pos, TokenType::SingleQuote)
                }
                '\'' if self.in_single_quote => {
                    self.in_single_quote = false;
                    self.create_token(pos, TokenType::SingleQuote)
                }
                ':' if !in_quote => self.create_token(pos, TokenType::Colon),
                '?' if !in_quote => self.create_token(pos, TokenType::QuestionMark),
                '-' if !in_quote => self.create_token(pos, TokenType::Dash),
                '{' if !in_quote => self.create_token(pos, TokenType::LeftBraket),
                '}' if !in_quote => self.create_token(pos, TokenType::RightBraket),
                '[' if !in_quote => self.create_token(pos, TokenType::LeftSqBraket),
                ']' if !in_quote => self.create_token(pos, TokenType::RightSqBraket),
                '\n' => self.create_token(pos, TokenType::Newline),
                _ => None, //TODO: Continue here
            },
            _ => None,
        }
    }

    fn create_indent(&mut self, pos: Position) -> Option<Token> {
        let mut count = 1;
        while let Some(c) = self.stream.next() {
            if c != ' ' {
                self.peeked_char = Some(c);
                break;
            }

            count += 1;
        }

        self.create_token(pos, TokenType::Spaces(count))
    }

    fn create_token(&self, pos: Position, content: TokenType<'a>) -> Option<Token> {
        Some(Token::new(pos, content))
    }
}

#[cfg(test)]
mod test_scanner {
    use super::*;

    #[test]
    fn test_empty() {
        let data = "";
        let mut scanner = Scanner::new(data);

        let token = scanner.next();
        assert!(token.is_none());
    }

    #[test]
    fn test_string() {
        let data = "Hello world!";
        let mut scanner = Scanner::new(data);

        let token = scanner.next();
        assert_token(token, 0, 0, 0, TokenType::String("Hello world!"));
    }

    fn assert_token(
        token: Option<Token>,
        offset: usize,
        line: usize,
        col: usize,
        content: TokenType,
    ) {
        assert!(token.is_some());
        let token = token.unwrap();
        assert_eq!(
            token,
            Token {
                pos: Position { offset, col, line },
                content
            },
            "Token does not match given position or content!"
        );
    }
}
