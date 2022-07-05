use crate::lexer::Token;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum OpCodeType {
    Add,
    Sub,
    ShiftLeft,
    ShiftRight,
    JmpZero,
    JmpNotZero,
    InputChar,
    PrintChar,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct OpCode {
    pub ty: OpCodeType,
    pub data: usize,
}

impl OpCode {
    pub fn new(ty: OpCodeType, data: usize) -> Self {
        Self { ty, data }
    }

    pub fn from_token(token: Token, data: usize) -> Self {
        let ty = match token {
            Token::Plus => OpCodeType::Add,
            Token::Minus => OpCodeType::Sub,
            Token::Less => OpCodeType::ShiftLeft,
            Token::Greater => OpCodeType::ShiftRight,
            Token::LBracket => OpCodeType::JmpZero,
            Token::RBracket => OpCodeType::JmpNotZero,
            Token::Comma => OpCodeType::InputChar,
            Token::Dot => OpCodeType::PrintChar,
        };

        Self::new(ty, data)
    }

    #[inline(always)]
    pub fn to_tuple(&self) -> (OpCodeType, usize) {
        (self.ty, self.data)
    }

    #[allow(dead_code)]
    pub fn to_string(&self) -> String {
        let op = format!("{:?}", self.ty);
        format!("{:14} {}\n", op, self.data)
    }
}
