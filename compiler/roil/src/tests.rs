// vim:ts=4:sw=4:et:ai

use crate::lexer::{Lexer, Token};

#[derive(Debug, PartialEq)]
pub enum Ins {
    LoadAImm16(u16),
    Return,
}

pub fn compile_from_str(input: &str) -> Vec<Ins> {
    let mut l = Lexer::new_from_str(input);

    let next = l.next();
    match next {
        Some(Token::Char(ch)) if ch == '-' => {
            let next = l.next();
            match next {
                Some(Token::Number(n)) => {
                    vec![Ins::LoadAImm16((!n).wrapping_add(1) as u16), Ins::Return]
                }
                _ => vec![],
            }
        }
        Some(Token::Number(n)) => vec![Ins::LoadAImm16(n as u16), Ins::Return],
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
