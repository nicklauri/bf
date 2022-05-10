use crate::parser::{TokenData, TokenList};
use std::fmt::{self, Display};

pub fn parse(src: &str) -> TokenList {
    let lexer = Lexer::new(src);

    lexer.parse()
}

/*
   Brainf*ck tokens: +-<>[],.
*/
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Token {
    Plus,
    Minus,
    Less,
    Greater,
    LBracket,
    RBracket,
    Comma,
    Dot,
}

impl Token {
    pub fn from_u8(ch: u8) -> Option<Self> {
        Some(match ch {
            b'+' => Token::Plus,
            b'-' => Token::Minus,
            b'<' => Token::Less,
            b'>' => Token::Greater,
            b'[' => Token::LBracket,
            b']' => Token::RBracket,
            b',' => Token::Comma,
            b'.' => Token::Dot,
            _ => return None,
        })
    }

    pub fn is_loop_token(&self) -> bool {
        matches!(self, Self::LBracket | Self::RBracket)
    }

    pub fn is_groupable(&self) -> bool {
        !self.is_loop_token()
    }
}

#[derive(Debug)]
pub struct Lexer<'a> {
    src: &'a [u8],
    pos: usize,
    loc: TokenLoc,
}

impl<'a> Lexer<'a> {
    pub fn from_bytes(src: &'a [u8]) -> Self {
        Self {
            src,
            pos: 0,
            loc: TokenLoc::new(),
        }
    }

    pub fn new(src: &'a str) -> Self {
        Self::from_bytes(src.as_bytes())
    }

    pub fn get_location(&self) -> TokenLoc {
        self.loc
    }

    pub fn parse(mut self) -> TokenList {
        self.src
            .iter()
            .filter_map(|&ch| self.get_token_with_location(ch))
            .collect()
    }

    fn inc_pos(&mut self) {
        self.pos += 1;
    }

    fn get_token_with_location(&mut self, ch: u8) -> Option<TokenData> {
        self.inc_pos();
        self.loc.update_location(ch);

        Token::from_u8(ch).map(|tok| (tok, self.get_location()))
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct TokenLoc {
    col: usize,
    line: usize,
}

impl TokenLoc {
    fn new() -> Self {
        Self { col: 0, line: 1 }
    }

    pub fn update_location(&mut self, ch: u8) {
        match ch {
            b'\n' => self.inc_line(),
            _ => self.inc_col(),
        }
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn col(&self) -> usize {
        self.col
    }

    pub(crate) fn from_col_line(col: usize, line: usize) -> Self {
        Self { line, col }
    }

    fn inc_line(&mut self) {
        self.line += 1;
        self.col = 0;
    }

    fn inc_col(&mut self) {
        self.col += 1;
    }
}

impl Display for TokenLoc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line(), self.col())
    }
}

#[cfg(test)]
mod test {
    use super::{Token::*, *};

    #[test]
    fn simple_text_one_line() {
        let input_string = "-fa+[d[df<>!!()*";
        let tokens = super::parse(input_string);

        let expected = vec![
            (Minus, TokenLoc::from_col_line(1, 1)),
            (Plus, TokenLoc::from_col_line(4, 1)),
            (LBracket, TokenLoc::from_col_line(5, 1)),
            (LBracket, TokenLoc::from_col_line(7, 1)),
            (Less, TokenLoc::from_col_line(10, 1)),
            (Greater, TokenLoc::from_col_line(11, 1)),
        ];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn simple_text_multi_lines() {
        let input_string = "\n-fa+[d\n[\ndf<>!\n!()*";
        let tokens = super::parse(input_string);

        let expected = vec![
            (Minus, TokenLoc::from_col_line(1, 2)),
            (Plus, TokenLoc::from_col_line(4, 2)),
            (LBracket, TokenLoc::from_col_line(5, 2)),
            (LBracket, TokenLoc::from_col_line(1, 3)),
            (Less, TokenLoc::from_col_line(3, 4)),
            (Greater, TokenLoc::from_col_line(4, 4)),
        ];

        assert_eq!(tokens, expected);
    }
}
