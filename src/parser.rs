/*
 *  Parser emits bytecodes for the VM.
 */

use anyhow::Result;

use crate::{
    lexer::{Token, TokenLoc},
    opcodes::OpCode,
};

pub type TokenData = (Token, TokenLoc);
pub type TokenList = Vec<TokenData>;
pub type Program = Vec<OpCode>;

pub fn parse(token_list: TokenList) -> Result<Program> {
    let parser = Parser::new(token_list);

    parser.parse()
}

#[derive(Debug)]
pub struct Parser {
    src: TokenList,
    src_pos: usize,
    lbracket_locations: Vec<(TokenLoc, usize)>,
    opcode_count: usize,
    program: Program,
}

impl Parser {
    pub fn new(src: TokenList) -> Self {
        Self {
            src,
            src_pos: 0,
            lbracket_locations: vec![],
            opcode_count: 0,
            program: Program::new(),
        }
    }

    pub fn parse(mut self) -> Result<Program> {
        // It is important to push each opcode into self.program instead of using iterator to collect all.
        // Method self.emit_jump_not_zero_data needs to patch JmpZero data.
        while let Some(op) = self.emit_opcode()? {
            self.program.push(op);
        }

        Ok(self.program)
    }

    pub fn next_token(&mut self) -> Option<TokenData> {
        let token_data = self.src.get(self.src_pos).map(|&data| data);

        if token_data.is_some() {
            self.increase_src_pos();
        }

        token_data
    }

    pub fn peek_token(&self) -> Option<TokenData> {
        self.src.get(self.src_pos).map(|&data| data)
    }

    pub fn emit_opcode(&mut self) -> Result<Option<OpCode>> {
        if let Some((token, location)) = self.next_token() {
            let data = match token {
                Token::LBracket => self.register_jump_not_zero_data(location),
                Token::RBracket => self.emit_jump_not_zero_data(location)?,
                _ => self.count_current_token(token),
            };

            self.opcode_count += 1;

            Ok(Some(OpCode::from_token(token, data)))
        } else if let Some((last_lbracket_location, _)) = self.lbracket_locations.last() {
            Err(self.emit_error_no_rbracket(last_lbracket_location))
        } else {
            Ok(None)
        }
    }

    pub fn count_current_token(&mut self, current_token: Token) -> usize {
        let mut counter = 1;

        if current_token.is_groupable() {
            while let Some((token, _)) = self.peek_token() {
                if token == current_token {
                    counter += 1;
                    self.increase_src_pos();
                } else {
                    break;
                }
            }
        }

        counter
    }

    pub fn register_jump_not_zero_data(&mut self, location: TokenLoc) -> usize {
        self.lbracket_locations.push((location, self.opcode_count));

        usize::MAX
    }

    pub fn emit_jump_not_zero_data(&mut self, location: TokenLoc) -> Result<usize> {
        if let Some((_, lbracket_idx)) = self.lbracket_locations.pop() {
            // Update lbracket JmpZero data.
            self.program[lbracket_idx].data = self.opcode_count;

            Ok(lbracket_idx)
        } else {
            Err(anyhow::anyhow!(
                "unexpected closing delimiter ']' at {}",
                location
            ))
        }
    }

    pub fn increase_src_pos(&mut self) {
        self.src_pos += 1;
    }

    pub fn emit_error_no_rbracket(&self, last_lbracket_location: &TokenLoc) -> anyhow::Error {
        let remaining_lbrackets = self.lbracket_locations.len();

        let extended_err_msg = if remaining_lbrackets > 1 {
            format!(" There are {} unclosed delimiters.", remaining_lbrackets)
        } else {
            String::new()
        };

        anyhow::anyhow!(
            "unclosed delimiter '[' at {}.{}",
            last_lbracket_location,
            extended_err_msg
        )
    }
}

#[cfg(test)]
mod test {
    use crate::{
        lexer::Lexer,
        opcodes::{OpCode, OpCodeType::*},
        parser::Parser,
    };

    #[test]
    fn parse_error_lbracket() {
        let token_list = Lexer::new("+[[]..").parse();

        let parse_result = Parser::new(token_list).parse();

        assert!(parse_result.is_err(), "parse_result must be Err at 2:1");
    }

    #[test]
    fn parse_error_rbracket() {
        let token_list = Lexer::new("+[..").parse();

        let parse_result = Parser::new(token_list).parse();

        assert!(parse_result.is_err(), "parse_result must be Err at 2:1");
    }

    #[test]
    fn parse_error_nested() {
        // input: [ [] [ [] ]
        // error: ^
        let token_list = Lexer::new("[ [] [ [] ]").parse();
        let parse_result = Parser::new(token_list).parse();

        assert!(
            parse_result.is_err(),
            "parse_result must be Err at 1:1\nparser_result={:#?}",
            parse_result.unwrap()
        );
    }

    #[test]
    fn stacked_opcodes() {
        let token_list = Lexer::new("+++>>>>[[[--]]]").parse();
        let parse_result = Parser::new(token_list).parse();

        assert!(parse_result.is_ok(), "parser_result={:#?}", parse_result);

        let program = parse_result.unwrap();

        let opcodes = vec![
            OpCode::new(Add, 3),
            OpCode::new(ShiftRight, 4),
            OpCode::new(JmpZero, 8),
            OpCode::new(JmpZero, 7),
            OpCode::new(JmpZero, 6),
            OpCode::new(Sub, 2),
            OpCode::new(JmpNotZero, 4),
            OpCode::new(JmpNotZero, 3),
            OpCode::new(JmpNotZero, 2),
        ];

        assert_eq!(program, opcodes);
    }

    #[test]
    fn parse_simple() {}

    #[test]
    fn parse_nested() {}
}
