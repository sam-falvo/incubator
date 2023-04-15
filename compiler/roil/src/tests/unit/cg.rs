// vim:ts=4:sw=4:et:ai

use crate::codegen::{cg_item, CtrlDest, DataDest, Ins, RegCache};
use crate::parser::Item;
use crate::symtab::SymTab;

#[test]
fn local_declarations() {
    let mut st = SymTab::new();
    st.create_local(&"x".to_string());
    let result = cg_item(
        Item::DeclareLocal("x".to_string(), Box::new(Item::ConstInteger(42))),
        &mut st,
        DataDest::RegA,
        CtrlDest::Return,
        &mut RegCache::new(),
    );
    assert_eq!(
        result,
        Ok(vec![Ins::LoadAImm16(42), Ins::StoreADP(2), Ins::Return,])
    );
}

#[test]
fn integers() {
    let result = cg_item(
        Item::ConstInteger(42),
        &mut SymTab::new(),
        DataDest::RegA,
        CtrlDest::Return,
        &mut RegCache::new(),
    );
    assert_eq!(result, Ok(vec![Ins::LoadAImm16(42), Ins::Return,]));
}

#[test]
fn local_access() {
    let result = cg_item(
        Item::LocalVar(24),
        &mut SymTab::new(),
        DataDest::RegA,
        CtrlDest::Return,
        &mut RegCache::new(),
    );
    assert_eq!(result, Ok(vec![Ins::LoadADP(24), Ins::Return,]));
}

#[test]
fn statement_list() {
    let result = cg_item(
        Item::StatementList(vec![
            Item::LocalVar(42),
            Item::Assign(
                Box::new(Item::LocalVar(36)),
                Box::new(Item::ConstInteger(99)),
            ),
            Item::LocalVar(24),
        ]),
        &mut SymTab::new(),
        DataDest::RegA,
        CtrlDest::Return,
        &mut RegCache::new(),
    );
    assert_eq!(
        result,
        Ok(vec![
            Ins::LoadAImm16(99),
            Ins::StoreADP(36),
            Ins::LoadADP(24),
            Ins::Return,
        ])
    );
}

#[test]
fn assignment() {
    let result = cg_item(
        Item::Assign(
            Box::new(Item::LocalVar(36)),
            Box::new(Item::ConstInteger(99)),
        ),
        &mut SymTab::new(),
        DataDest::RegA,
        CtrlDest::Return,
        &mut RegCache::new(),
    );
    assert_eq!(
        result,
        Ok(vec![Ins::LoadAImm16(99), Ins::StoreADP(36), Ins::Return,])
    );
}

#[test]
fn add_sub() {
    let result = cg_item(
        Item::Add(
            Box::new(Item::LocalVar(2)),
            Box::new(Item::Sub(
                Box::new(Item::LocalVar(4)),
                Box::new(Item::LocalVar(6)),
            )),
        ),
        &mut SymTab::new(),
        DataDest::RegA,
        CtrlDest::Return,
        &mut RegCache::new(),
    );
    assert_eq!(
        result,
        Ok(vec![
            Ins::LoadADP(4),
            Ins::SubtractADP(6),
            Ins::AddADP(2),
            Ins::Return,
        ])
    );
}
