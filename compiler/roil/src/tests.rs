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

    match p.next {
        Some(Token::Let) => {
            p.skip();

            eprintln!("GOT LET");
            let id: String;
            eprintln!("  {:?}", p.next);
            if let Some(Token::Id(s1)) = p.next.clone() {
                id = s1;
                p.skip();
                eprintln!("GOT ID: {:?}", id);
            } else {
                return vec![];
            }

            if let Some(Token::Char(':')) = p.next {
                p.skip();
                eprintln!("GOT :");
            } else {
                return vec![];
            }

            let type_: String;
            if let Some(Token::Id(s2)) = p.next.clone() {
                type_ = s2;
                p.skip();
                eprintln!("GOT TYPE: {:?}", type_);
            } else {
                return vec![];
            }

            let rval;
            if let Some(Token::Char('=')) = p.next {
                p.skip();
                eprintln!("GOT =");

                rval = p.g_expr();
            } else {
                return vec![];
            }

            if let Some(Token::Char(';')) = p.next {
                p.skip();
                eprintln!("GOT ;");
            } else {
                return vec![];
            }

            eprintln!("  LET {:?} : {:?} = {:?}", id, type_, rval);
            eprintln!("Let Not implemented");
            vec![]
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
    let dp_base = 2;

    let result = compile_from_str("let x: u16 = 0;");
    assert_eq!(result, vec![Ins::LoadAImm16(0), Ins::StoreADP(dp_base), Ins::Return,]);

    let result = compile_from_str("let x: u16 = 1; let y: u16 = 2;");
    assert_eq!(result, vec![
        Ins::LoadAImm16(1), Ins::StoreADP(dp_base),
        Ins::LoadAImm16(2), Ins::StoreADP(dp_base+2),
        Ins::Return,
    ]);
}
