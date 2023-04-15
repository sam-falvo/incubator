// vim:ts=4:sw=4:et:ai

use crate::codegen::{cg_item, CtrlDest, DataDest, Ins, RegCache};
use crate::parser::{ErrType, Parser};
use crate::symtab::SymTab;

pub fn compile_from_str(input: &str) -> Result<Vec<Ins>, ErrType> {
    let mut p = Parser::new(input);
    let mut st = SymTab::new();
    let mut rc_a = RegCache::new();

    let item = p.g_statement(&mut st);
    cg_item(item, &mut st, DataDest::RegA, CtrlDest::Return, &mut rc_a)
}

#[test]
fn integers() {
    let result = compile_from_str("42");
    assert_eq!(result, Ok(vec![Ins::LoadAImm16(42), Ins::Return,]));

    let result = compile_from_str("420");
    assert_eq!(result, Ok(vec![Ins::LoadAImm16(420), Ins::Return,]));

    let result = compile_from_str("49_152");
    assert_eq!(result, Ok(vec![Ins::LoadAImm16(49152), Ins::Return,]));

    let result = compile_from_str("0xC000");
    assert_eq!(result, Ok(vec![Ins::LoadAImm16(49152), Ins::Return,]));

    let result = compile_from_str("0q10");
    assert_eq!(result, Ok(vec![Ins::LoadAImm16(8), Ins::Return,]));

    // I really hate C-style octal syntax.  AT&T should be ashamed of
    // themselves and should feel bad.
    let result = compile_from_str("010");
    assert_eq!(result, Ok(vec![Ins::LoadAImm16(8), Ins::Return,]));

    let result = compile_from_str("080");
    assert_eq!(result, Ok(vec![Ins::LoadAImm16(80), Ins::Return,]));

    // Numbers formatted like 0018 or 018 will not (currently) parse correctly.
    // This is a known issue; but, fixing this is low priority as I write this.

    let result = compile_from_str("-42");
    assert_eq!(result, Ok(vec![Ins::LoadAImm16(0xFFD6), Ins::Return,]));
}

#[test]
fn let_binding() {
    let dp_result = 0;
    let dp_locals = dp_result + 2;

    let result = compile_from_str("let x = 0");
    assert_eq!(result, Ok(vec![Ins::StoreZeroDP(dp_locals), Ins::Return,]));

    let result = compile_from_str("begin let x = 0");
    assert_eq!(result, Ok(vec![Ins::StoreZeroDP(dp_locals), Ins::Return,]));

    let result = compile_from_str("begin let x = 1; let y=2");
    assert_eq!(
        result,
        Ok(vec![
            Ins::LoadAImm16(1),
            Ins::StoreADP(dp_locals),
            Ins::LoadAImm16(2),
            Ins::StoreADP(dp_locals + 2),
            Ins::Return,
        ])
    );
}

#[test]
fn expressions() {
    let dp_result = 0;
    let dp_locals = dp_result + 2;

    let result = compile_from_str("begin let x=1; let y=2; x+y-2");
    assert_eq!(
        result,
        Ok(vec![
            Ins::LoadAImm16(1),
            Ins::StoreADP(dp_locals),
            Ins::LoadAImm16(2),
            Ins::StoreADP(dp_locals + 2),
            Ins::AddADP(dp_locals),
            Ins::DecA,
            Ins::DecA,
            Ins::Return,
        ])
    );
}

#[test]
fn assignments() {
    let dp_result = 0;
    let dp_locals = dp_result + 2;

    let result = compile_from_str("begin let x=0; x: x + 20");
    assert_eq!(
        result,
        Ok(vec![
            Ins::StoreZeroDP(dp_locals),
            Ins::LoadADP(dp_locals),
            Ins::AddAImm16(20),
            Ins::StoreADP(dp_locals),
            Ins::Return,
        ])
    );

    let result = compile_from_str("begin let x=0; let y=0; y: x: x + 20");
    assert_eq!(
        result,
        Ok(vec![
            Ins::StoreZeroDP(dp_locals),
            Ins::StoreZeroDP(dp_locals + 2),
            Ins::LoadADP(dp_locals),
            Ins::AddAImm16(20),
            Ins::StoreADP(dp_locals),
            Ins::StoreADP(dp_locals + 2),
            Ins::Return,
        ])
    );
}

mod symbol_table {
    use crate::symtab::Errors;
    use crate::symtab::SymTab;

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
