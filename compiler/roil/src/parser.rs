// vim:ts=4:sw=4:et:ai

use crate::lexer::{Lexer, Token};
use crate::symtab::SymTab;

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

    pub fn g_expr(&mut self, st: &SymTab) -> Item {
        self.g_sum(st)
    }

    pub fn g_sum(&mut self, st: &SymTab) -> Item {
        let mut lhs = self.g_prod(st);

        if let Item::Error(_) = lhs {
            return lhs;
        }

        loop {
            match self.next {
                Some(Token::Char('+')) => {
                    self.skip();
                    let rhs = self.g_prod(st);
                    if let Item::Error(_) = rhs {
                        return rhs;
                    }
                    lhs = Item::Add(Box::new(lhs), Box::new(rhs));
                }

                Some(Token::Char('-')) => {
                    self.skip();
                    let rhs = self.g_prod(st);
                    if let Item::Error(_) = rhs {
                        return rhs;
                    }
                    lhs = Item::Sub(Box::new(lhs), Box::new(rhs));
                }

                _ => return lhs,
            }
        }
    }

    pub fn g_prod(&mut self, st: &SymTab) -> Item {
        self.g_unary(st)
    }

    pub fn g_unary(&mut self, st: &SymTab) -> Item {
        match self.next {
            Some(Token::Char('-')) => {
                self.skip();
                let e = self.g_primary(st);
                negate(e)
            }

            _ => self.g_primary(st),
        }
    }

    pub fn g_primary(&mut self, st: &SymTab) -> Item {
        match self.next {
            Some(Token::Number(n)) => {
                let i = Item::ConstInteger(n as TargetUInt);
                self.skip();
                i
            }

            Some(Token::Id(ref name)) => {
                let id_or_err = st.find_by_name(&name);
                let prim = match id_or_err {
                    Err(_) => Item::Error(ErrType::UndefinedId(name.to_string())),
                    Ok(sym) => Item::LocalVar(sym.offset as TargetByte),
                };
                self.skip();
                if let Some(Token::Char(':')) = self.next {
                    self.skip();
                    let rhs = self.g_sum(st);

                    if let Item::LocalVar(_) = prim {
                        Item::Assign(Box::new(prim), Box::new(rhs))
                    } else {
                        Item::Error(ErrType::LValExpected)
                    }
                } else {
                    prim
                }
            }

            _ => Item::Error(ErrType::PrimaryExpected),
        }
    }

    pub fn g_statement(&mut self, st: &mut SymTab) -> Item {
        match self.next {
            Some(Token::Let) => {
                self.skip();
                self.g_let(st)
            }

            Some(Token::Begin) => {
                self.skip();
                self.g_statement_block(st)
            }

            _ => self.g_expr(st),
        }
    }

    pub fn g_statement_block(&mut self, st: &mut SymTab) -> Item {
        let mut block: Vec<Item> = Vec::new();
        loop {
            let s: Item = self.g_statement(st);
            let s1 = s.clone();
            if let Item::Error(_) = s1 {
                return s;
            } else {
                block.push(s);
            }
            if self.next != Some(Token::Char(';')) {
                break;
            }
            self.skip();
        }
        Item::StatementList(block)
    }

    pub fn g_let(&mut self, st: &mut SymTab) -> Item {
        // Parse "let <id> = <expr>"
        // The 'let' token was already consumed.

        let id: String;
        if let Some(Token::Id(s1)) = self.next.clone() {
            id = s1;
            self.skip();
        } else {
            return Item::Error(ErrType::IdentifierExpected);
        }

        let rval;
        if let Some(Token::Char('=')) = self.next {
            self.skip();
            rval = self.g_expr(st);
        } else {
            return Item::Error(ErrType::CharExpected('='));
        }

        st.create_local(&id);
        Item::DeclareLocal(id, Box::new(rval))
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum ErrType {
    IdentifierExpected,
    CharExpected(char),
    UndefinedId(String),
    PrimaryExpected,
    LValExpected,

    // These tend to be used by the code generator.
    ExpressionExpected,
    ParserNotFoldingConstants,
    UnexpectedCGArgs,
    UnexpectedApplyOp,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Item {
    Error(ErrType),
    ConstInteger(TargetUInt),
    DeclareLocal(String, Box<Item>),
    LocalVar(TargetByte),
    StatementList(Vec<Item>),
    Add(Box<Item>, Box<Item>),
    Sub(Box<Item>, Box<Item>),
    Assign(Box<Item>, Box<Item>),
    Apply(Type, Op, Box<Item>, Box<Item>),
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Type {
    // Unsigned 16-bit integer
    Cardinal,
    // Cardinal-sized bit set
    BitSet,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Op {
    Add,
    Subtract,
}
