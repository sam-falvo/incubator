// vim:ts=4:sw=4:et:ai

use crate::parser::{Item, Parser, TargetUInt, TargetByte};
use crate::lexer::Token;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Ins {
    LoadAImm16(TargetUInt),
    StoreADP(TargetByte),
    Return,
}

pub fn compile_from_str(input: &str) -> Vec<Ins> {
    let mut p = Parser::new(input);
    let mut st = SymTab::new();

    match p.next {
        Some(Token::Let) => {
            p.skip();

            let id: String;
            if let Some(Token::Id(s1)) = p.next.clone() {
                id = s1;
                p.skip();
            } else {
                return vec![];
            }

            if let Some(Token::Char(':')) = p.next {
                p.skip();
            } else {
                return vec![];
            }

            let type_: String;
            if let Some(Token::Id(s2)) = p.next.clone() {
                type_ = s2;
                p.skip();
            } else {
                return vec![];
            }

            let rval;
            if let Some(Token::Char('=')) = p.next {
                p.skip();
                rval = p.g_expr();
            } else {
                return vec![];
            }

            if let Some(Token::Char(';')) = p.next {
                p.skip();
            } else {
                return vec![];
            }

            eprintln!("  LET {:?} : {:?} = {:?}", id, type_, rval);

            st.create_local(&id);
            match st.find_by_name(&id) {
                Ok(sym) => {
                    match rval {
                        Item::ConstInteger(n) => vec![Ins::LoadAImm16(n), Ins::StoreADP(sym.offset as u8), Ins::Return,],
                        _ => vec![],
                    }
                }
                Err(_) => vec![],
            }
        }

        _ => {
            let i = p.g_expr();
            match i {
                Item::ConstInteger(n) => vec![Ins::LoadAImm16(n), Ins::Return],
                _ => vec![],
            }
        }
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

    // Numbers formatted like 0018 or 018 will not (currently) parse correctly.
    // This is a known issue; but, fixing this is low priority as I write this.

    let result = compile_from_str("-42");
    assert_eq!(result, vec![Ins::LoadAImm16(0xFFD6), Ins::Return,]);
}

#[test]
fn let_binding() {
    let dp_result = 0;
    let dp_locals = dp_result + 2;

    let result = compile_from_str("let x: u16 = 0;");
    assert_eq!(result, vec![Ins::LoadAImm16(0), Ins::StoreADP(dp_locals), Ins::Return,]);

    let result = compile_from_str("let x: u16 = 1; let y: u16 = 2;");
    assert_eq!(result, vec![
        Ins::LoadAImm16(1), Ins::StoreADP(dp_locals),
        Ins::LoadAImm16(2), Ins::StoreADP(dp_locals+2),
        Ins::Return,
    ]);
}


// Using struct-of-arrays helps avoid the need for complicated reference types
// and pesky lifetime annotations.
pub struct SymTab {
    length: usize,
    names: Vec<String>,
    offsets: Vec<u16>,
    next_local: u16,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Symbol<'name_lifetime> {
    pub name: &'name_lifetime str,
    pub offset: u16,
}

impl SymTab {
    pub fn new() -> Self {
        SymTab {
            length: 0,
            names: Vec::new(),
            offsets: Vec::new(),
            next_local: 2,
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn create_local(&mut self, name: &str) {
        self.names.push(name.to_string());
        self.offsets.push(self.next_local);
        self.length += 1;
        self.next_local += 2;
    }

    pub fn find_by_name<'n>(&'n self, name: &'n str) -> Result<Symbol, Errors> {
        // If the symbol table is empty, then, by definition,
        // the symbol cannot be found.
        if self.length == 0 {
            return Err(Errors::Undefined);
        }

        // The current index into the name and offset vectors.
        //
        // Since the last inserted symbol will appear at the end of the vector,
        // and since new symbols take priority over shadowed symbols, we start
        // our scan at the *end* of the name vector.
        let mut i: usize = self.length - 1;

        // Try to find the symbol, starting from the tail of the vector and
        // working our way back to the beginning.
        loop {
            let candidate = self.names.get(i).ok_or(Errors::BadSymbolTableIndex)?;
            if candidate.as_str() == name {
                break;
            }

            // Candidate is still on the loose.  We need to look at the next name
            // in the vector.  However, if we're already at the beginning, then
            // we've exhausted the search space.  Give up.
            if i == 0 {
                return Err(Errors::Undefined);
            }

            i -= 1;
        }

        // If we're here, then we must have found a viable candidate.
        return Ok(Symbol{
            name: name,
            offset: *self.offsets.get(i).ok_or(Errors::BadSymbolTableIndex)?,
        });
    }
}

impl Symbol<'_> {
    pub fn offset(&self) -> u16 {
        self.offset
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Errors {
    Undefined,
    BadSymbolTableIndex,
}

mod symbol_table {
    use super::SymTab;
    use super::Errors;

    #[test]
    fn new_creates_empty_table() {
        let s = SymTab::new();
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn find_by_bad_name() {
        let s = SymTab::new();
        assert_eq!(s.find_by_name("undefined"), Err(Errors::Undefined));
    }

    #[test]
    fn create_local() {
        let mut s = SymTab::new();
        s.create_local("x");
        assert_eq!(s.find_by_name("x").unwrap().offset(), 2);

        s.create_local("y");
        assert_eq!(s.find_by_name("y").unwrap().offset(), 4);

        // Shadow support
        s.create_local("x");
        assert_eq!(s.find_by_name("x").unwrap().offset(), 6);
    }
}
