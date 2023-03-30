use crate::lexer::{Lexer, Token};

pub type TargetSInt = i16;
pub type TargetUInt = u16;
pub type TargetByte = u8;

pub struct Parser<'input_lifetime> {
    lexer: Lexer<'input_lifetime>,
    pub next: Option<Token>,
}

fn negate(i: Item) -> Item {
    match i {
        Item::ConstInteger(n) => Item::ConstInteger(-(n as TargetSInt) as TargetUInt),
        _ => i,
    }
}

impl<'input_lifetime> Parser<'input_lifetime> {
    pub fn new(input: &'input_lifetime str) -> Self {
        let mut p = Self {
            lexer: Lexer::new_from_str(input),
            next: None,
        };
        p.skip(); // prime the token stream

        p
    }

    pub fn skip(&mut self) {
        self.next = self.lexer.next();
    }

    pub fn g_expr(&mut self) -> Item {
        self.g_unary()
    }

    pub fn g_statement(&mut self) -> Item {
        match self.next {
            Some(Token::Let) => {
                self.skip();
                self.g_let()
            }

            _ => self.g_expr(),
        }
    }

    pub fn g_let(&mut self) -> Item {
        // Parse "let <id> : u16 = <expr>"
        // The 'let' token was already consumed.

        let id: String;
        if let Some(Token::Id(s1)) = self.next.clone() {
            id = s1;
            self.skip();
        } else {
            return Item::Error;
        }

        if let Some(Token::Char(':')) = self.next {
            self.skip();
        } else {
            return Item::Error;
        }

        if let Some(Token::Id(s2)) = self.next.clone() {
            self.skip();
            if s2.as_str() != "u16" {
                return Item::Error;
            }
        } else {
            return Item::Error;
        }

        let rval;
        if let Some(Token::Char('=')) = self.next {
            self.skip();
            rval = self.g_expr();
        } else {
            return Item::Error;
        }

        Item::DeclareLocal(id, Box::new(rval))
    }

    pub fn g_unary(&mut self) -> Item {
        match self.next {
            Some(Token::Char('-')) => {
                self.skip();
                let e = self.g_primary();
                negate(e)
            }

            _ => self.g_primary(),
        }
    }

    pub fn g_primary(&mut self) -> Item {
        match self.next {
            Some(Token::Number(n)) => {
                let i = Item::ConstInteger(n as TargetUInt);
                self.skip();
                i
            }
            _ => Item::Error,
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Item {
    Error,
    ConstInteger(TargetUInt),
    DeclareLocal(String, Box<Item>),
}
