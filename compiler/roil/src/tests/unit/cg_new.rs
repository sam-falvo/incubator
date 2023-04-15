// vim:ts=4:sw=4:et:ai

use crate::codegen::{cg_item, CtrlDest, DataDest, Ins, RegCache};
use crate::parser::Item;
use crate::parser::Op;
use crate::parser::Type;
use crate::symtab::SymTab;

#[test]
fn add_sub() {
    // (a-b)+(c-d)
    let result = cg_item(
        Item::Apply(
            Type::Cardinal,
            Op::Add,
            Box::new(Item::Apply(
                Type::Cardinal,
                Op::Subtract,
                Box::new(Item::LocalVar(12)),
                Box::new(Item::ConstInteger(100)),
            )),
            Box::new(Item::Apply(
                Type::Cardinal,
                Op::Subtract,
                Box::new(Item::LocalVar(14)),
                Box::new(Item::LocalVar(16)),
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
            Ins::LoadADP(12),
            Ins::SubtractAImm16(100),
            Ins::StoreADP(2),
            Ins::LoadADP(14),
            Ins::SubtractADP(16),
            Ins::AddADP(2),
            Ins::Return,
        ])
    );

    // (a+b)-(c+d)
    let result = cg_item(
        Item::Apply(
            Type::Cardinal,
            Op::Subtract,
            Box::new(Item::Apply(
                Type::Cardinal,
                Op::Add,
                Box::new(Item::LocalVar(12)),
                Box::new(Item::ConstInteger(100)),
            )),
            Box::new(Item::Apply(
                Type::Cardinal,
                Op::Add,
                Box::new(Item::LocalVar(14)),
                Box::new(Item::LocalVar(16)),
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
            Ins::LoadADP(16),
            Ins::AddADP(14),
            Ins::StoreADP(2),
            Ins::LoadADP(12),
            Ins::AddAImm16(100),
            Ins::SubtractADP(2),
            Ins::Return,
        ])
    );
}
