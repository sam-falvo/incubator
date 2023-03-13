// vim:ts=4:sw=4:et:ai

use crate::lexer::{Lexer, Token};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Ins {
    LoadAImm16(u16),
    Return,
}

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
            Item::ConstInteger(n) => Item::ConstInteger((!n).wrapping_add(1)),
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
                let i = Item::ConstInteger(n as u16);
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
    ConstInteger(u16),
}

pub fn compile_from_str(input: &str) -> Vec<Ins> {
    let mut p = Parser::new(input);
    let i = p.g_expr();
    match i {
        Item::ConstInteger(n) => vec![Ins::LoadAImm16(n), Ins::Return],
        _ => vec![],
    }
}

#[test]
fn unsigned_integer() {
    let result = compile_from_str("42");
    assert_eq!(result, vec![Ins::LoadAImm16(42), Ins::Return,]);

    let result = compile_from_str("420");
    assert_eq!(result, vec![Ins::LoadAImm16(420), Ins::Return,]);

    let result = compile_from_str("49_152");
    assert_eq!(result, vec![Ins::LoadAImm16(49152), Ins::Return,]);

    let result = compile_from_str("0xC000");
    assert_eq!(result, vec![Ins::LoadAImm16(49152), Ins::Return,]);

    let result = compile_from_str("0q10");
    assert_eq!(result, vec![Ins::LoadAImm16(8), Ins::Return,]);

    // I really hate C-style octal syntax.  AT&T should be ashamed of
    // themselves and should feel bad.
    let result = compile_from_str("010");
    assert_eq!(result, vec![Ins::LoadAImm16(8), Ins::Return,]);

    let result = compile_from_str("080");
    assert_eq!(result, vec![Ins::LoadAImm16(80), Ins::Return,]);

    let result = compile_from_str("-42");
    assert_eq!(result, vec![Ins::LoadAImm16(0xFFD6), Ins::Return,]);
}
