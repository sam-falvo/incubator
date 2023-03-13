use crate::lexer::{Lexer, Token};

pub type TargetSInt = i16;
pub type TargetUInt = u16;

pub struct Parser<'input_lifetime> {
    lexer: Lexer<'input_lifetime>,
    pub next: Option<Token>,
}

impl<'input_lifetime> Parser<'input_lifetime> {
    pub fn new(input: &'input_lifetime str) -> Self {
        let mut p = Self { lexer: Lexer::new_from_str(input), next: None };
        p.skip(); // prime the token stream

        p
    }

    pub fn skip(&mut self) {
        self.next = self.lexer.next();
    }

    fn negate(&self, i: Item) -> Item { 
        match i {
            Item::ConstInteger(n) => Item::ConstInteger(-(n as TargetSInt) as TargetUInt),
            _ => i,
        }
    }

    pub fn g_expr(&mut self) -> Item {
        self.g_unary()
    }

    pub fn g_unary(&mut self) -> Item {
        match self.next {
            Some(Token::Char('-')) => {
                self.skip();
                let e = self.g_primary();
                self.negate(e)
            },

            _ => {
                self.g_primary()
            },
        }
    }

    pub fn g_primary(&mut self) -> Item {
        match self.next {
            Some(Token::Number(n)) => {
                let i = Item::ConstInteger(n as TargetUInt);
                self.skip();
                i
            },
            _ => Item::Error,
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Item {
    Error,
    ConstInteger(TargetUInt),
}
