// vim:ts=4:sw=4:et:ai

use crate::parser::{Item, TargetByte, TargetUInt, ErrType};
use crate::symtab::SymTab;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CtrlDest {
    Return,
    Next,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DataDest {
    Effect,
    RegA,
    Local(u8),
}


#[derive(Debug, PartialEq, Clone)]
pub enum Ins {
    LoadAImm16(TargetUInt),
    LoadADP(TargetByte),
    StoreADP(TargetByte),

    IncA,
    DecA,
    AddADP(TargetByte),
    AddAImm16(TargetUInt),
    SubtractADP(TargetByte),
    SubtractAImm16(TargetUInt),

    StoreZeroDP(TargetByte),

    Return,
}

// Tracks known attributes of the accumulator
pub struct RegCache {
    // If not None, we know the accumulator must have this value.
    value: Option<u16>,

    // If not None, we know the accumulator holds the value in the specified local.
    local: Option<TargetByte>,
}

impl RegCache {
    pub const fn new() -> Self {
        Self {
            value: None,
            local: None,
        }
    }
}

fn increment_a(listing: &mut Vec<Ins>, rc_a: &mut RegCache) {
    listing.push(Ins::IncA);
    if let Some(n) = rc_a.value {
        rc_a.value = Some(n.wrapping_add(1) as u16);
    }
    rc_a.local = None;
}

fn decrement_a(listing: &mut Vec<Ins>, rc_a: &mut RegCache) {
    listing.push(Ins::DecA);
    if let Some(n) = rc_a.value {
        rc_a.value = Some(n.wrapping_sub(1) as u16);
    }
    rc_a.local = None;
}

fn load_a_imm16(listing: &mut Vec<Ins>, rc_a: &mut RegCache, value: u16) {
    listing.push(Ins::LoadAImm16(value));
    rc_a.value = Some(value);
    rc_a.local = None;
}

fn load_a_local(listing: &mut Vec<Ins>, rc_a: &mut RegCache, offset: u8) {
    if let Some(n) = rc_a.local {
        if n == offset {
            // Already loaded into A; skip code generation.
            return;
        }
    }
    listing.push(Ins::LoadADP(offset));
    rc_a.value = None;
    rc_a.local = Some(offset);
}

fn store_zero_local(listing: &mut Vec<Ins>, rc_a: &mut RegCache, offset: u8) {
    listing.push(Ins::StoreZeroDP(offset));
    if Some(offset) == rc_a.local {
        rc_a.local = None;
    }
}

fn store_a_local(listing: &mut Vec<Ins>, rc_a: &mut RegCache, offset: u8) {
    listing.push(Ins::StoreADP(offset));
    rc_a.local = Some(offset);
}

fn add_a_imm16(listing: &mut Vec<Ins>, rc_a: &mut RegCache, value: u16) {
    listing.push(Ins::AddAImm16(value));
    if let Some(n) = rc_a.value {
        rc_a.value = Some(n.wrapping_add(value) as u16);
    }
    rc_a.local = None;
}

fn add_a_local(listing: &mut Vec<Ins>, rc_a: &mut RegCache, offset: u8) {
    listing.push(Ins::AddADP(offset));
    rc_a.value = None;
    rc_a.local = None;
}

fn subtract_a_imm16(listing: &mut Vec<Ins>, rc_a: &mut RegCache, value: u16) {
    listing.push(Ins::SubtractAImm16(value));
    if let Some(n) = rc_a.value {
        rc_a.value = Some(n.wrapping_sub(value) as u16);
    }
    rc_a.local = None;
}

fn subtract_a_local(listing: &mut Vec<Ins>, rc_a: &mut RegCache, offset: u8) {
    listing.push(Ins::SubtractADP(offset));
    rc_a.value = None;
    rc_a.local = None;
}


pub fn cg_const_int(n: u16, dd: DataDest, cd: CtrlDest, rc_a: &mut RegCache) -> Result<Vec<Ins>, ErrType> {
    let mut listing: Vec<Ins> = Vec::new();

    match dd {
        DataDest::RegA | DataDest::Effect => load_a_imm16(&mut listing, rc_a, n),
        DataDest::Local(ofs) => {
            if n == 0 {
                store_zero_local(&mut listing, rc_a, ofs);
            } else {
                load_a_imm16(&mut listing, rc_a, n);
                store_a_local(&mut listing, rc_a, ofs);
            }
        },
    }

    listing.extend_from_slice(&cg_goto(cd)?);
    Ok(listing)
}

pub fn cg_declare_local(st: &mut SymTab, id: String, rval: Item, _dd: DataDest, cd: CtrlDest, rc_a: &mut RegCache) -> Result<Vec<Ins>, ErrType> {
    let mut listing: Vec<Ins> = Vec::new();

    match st.find_by_name(&id) {
        Ok(sym) => {
            let offset = sym.offset as u8;
            listing.extend_from_slice(&cg_item(rval, st, DataDest::Local(offset), CtrlDest::Next, rc_a)?);
        }

        _ => return Err(ErrType::UndefinedId(id.clone())),
    }

    listing.extend_from_slice(&cg_goto(cd)?);
    Ok(listing)
}

pub fn cg_local(offset: u8, dd: DataDest, cd: CtrlDest, rc_a: &mut RegCache) -> Result<Vec<Ins>, ErrType> {
    let mut listing: Vec<Ins> = Vec::new();

    match dd {
        DataDest::RegA => load_a_local(&mut listing, rc_a, offset),
        DataDest::Effect => (),
        DataDest::Local(target) => {
            listing.extend_from_slice(&cg_local(offset, DataDest::RegA, CtrlDest::Next, rc_a)?);
            store_a_local(&mut listing, rc_a, target);
        }
    }

    listing.extend_from_slice(&cg_goto(cd)?);
    Ok(listing)
}

pub fn cg_statement_list(statements: Vec<Item>, st: &mut SymTab, dd: DataDest, cd: CtrlDest, rc_a: &mut RegCache) -> Result<Vec<Ins>, ErrType> {
    let mut listing: Vec<Ins> = Vec::new();
    let length = statements.len();

    if length > 0 {
        let last = length - 1;

        for i in 0..last {
            listing.extend_from_slice(&cg_item(statements[i].clone(), st, DataDest::Effect, CtrlDest::Next, rc_a)?);
        }
        listing.extend_from_slice(&cg_item(statements[last].clone(), st, dd, cd, rc_a)?);
    }

    Ok(listing)
}

pub fn cg_add(lhs: Box<Item>, rhs: Box<Item>, st: &mut SymTab, dd: DataDest, cd: CtrlDest, rc_a: &mut RegCache) -> Result<Vec<Ins>, ErrType> {
    let mut listing: Vec<Ins> = Vec::new();

    // const + const            ERROR: parser not folding constants
    // const + localVar         fetch local, add constant
    // const + add              calculate sum, add constant
    // const + sub              calculate diff, add constant
    // localVar + const         swap; recurse
    // localVar + localVar      fetch local, add local
    // localVar + add           calculate sum, add local
    // localVar + sub           calculate diff, add local
    // add + const              swap; recurse
    // add + localVar           swap; recurse
    // add + add                alloc temp; t = rhs; calc lhs; add t; free temp
    // add + sub                alloc temp; t = rhs; calc lhs; add t; free temp
    // sub + const              swap; recurse
    // sub + localVar           swap; recurse
    // sub + add                swap; recurse
    // sub + sub                alloc temp; t = rhs; calc lhs; add t; free temp

    // Take advantage of addition's commutative property to reduce the amount of
    // special-case code we need to implement.
    match *lhs {
        Item::ConstInteger(_) => match *rhs {
            Item::ConstInteger(_) => return Err(ErrType::ParserNotFoldingConstants),
            _ => (),
        }

        Item::LocalVar(_) => match *rhs {
            Item::ConstInteger(_) => return cg_add(rhs, lhs, st, dd, cd, rc_a),
            _ => (),
        }

        Item::Add(_, _) => match *rhs {
            Item::ConstInteger(_) => return cg_add(rhs, lhs, st, dd, cd, rc_a),
            Item::LocalVar(_) => return cg_add(rhs, lhs, st, dd, cd, rc_a),
            _ => (),
        }

        Item::Sub(_, _) => match *rhs {
            Item::ConstInteger(_) => return cg_add(rhs, lhs, st, dd, cd, rc_a),
            Item::LocalVar(_) => return cg_add(rhs, lhs, st, dd, cd, rc_a),
            Item::Add(_, _) => return cg_add(rhs, lhs, st, dd, cd, rc_a),
            _ => (),
        }

        _ => return Err(ErrType::UnexpectedCGArgs),
    }

    match *lhs {
        Item::ConstInteger(n) => match *rhs {
            Item::LocalVar(_) | Item::Add(_, _) | Item::Sub(_, _) => {
                listing.extend_from_slice(&cg_item(*rhs, st, DataDest::RegA, CtrlDest::Next, rc_a)?);
                if n == 1 {
                    increment_a(&mut listing, rc_a);
                } else if n == !0 {
                    decrement_a(&mut listing, rc_a);
                } else {
                    add_a_imm16(&mut listing, rc_a, n);
                }
            }

            _ => return Err(ErrType::UnexpectedCGArgs),
        }

        Item::LocalVar(offset) => match *rhs {
            Item::LocalVar(_) | Item::Add(_, _) | Item::Sub(_, _) => {
                listing.extend_from_slice(&cg_item(*rhs, st, DataDest::RegA, CtrlDest::Next, rc_a)?);
                add_a_local(&mut listing, rc_a, offset);
            }

            _ => return Err(ErrType::UnexpectedCGArgs),
        }

        Item::Add(_, _) | Item::Sub(_, _) => match *rhs {
            Item::Add(_, _) | Item::Sub(_, _) => {
                let t: u8 = st.alloc_temp() as u8;
                listing.extend_from_slice(&cg_item(*lhs, st, DataDest::Local(t), CtrlDest::Next, rc_a)?);
                listing.extend_from_slice(&cg_item(*rhs, st, DataDest::RegA, CtrlDest::Next, rc_a)?);
                add_a_local(&mut listing, rc_a, t as u8);
                st.free_temp();
            }

            _ => return Err(ErrType::UnexpectedCGArgs),
        }

        _ => return Err(ErrType::UnexpectedCGArgs),
    }

    listing.extend_from_slice(&cg_store_a(dd, rc_a)?);
    listing.extend_from_slice(&cg_goto(cd)?);
    Ok(listing)
}

pub fn cg_sub(lhs: Box<Item>, rhs: Box<Item>, st: &mut SymTab, dd: DataDest, cd: CtrlDest, rc_a: &mut RegCache) -> Result<Vec<Ins>, ErrType> {
    let mut listing: Vec<Ins> = Vec::new();

    // const - const            ERROR: parser not folding constants
    // const - localVar         load const, subtract local
    // const - add              alloc temp; t = rhs; load const; subtract t; free temp
    // const - sub              alloc temp; t = rhs; load const; subtract t; free temp
    // localVar - const         load local; subtract const
    // localVar - localVar      load local, subtract local
    // localVar - add           alloc temp; t = rhs; load local; subtract t; free temp
    // localVar - sub           alloc temp; t = rhs; load local; subtract t; free temp
    // add - const              gen lhs; subtract constant
    // add - localVar           gen lhs; subtract local
    // add - add                alloc temp; t = rhs; gen lhs; subtract t; free temp
    // add - sub                alloc temp; t = rhs; gen lhs; subtract t; free temp
    // sub - const              same as add-const
    // sub - localVar           same as add-local
    // sub - add                same as add-add
    // sub - sub                same as add-sub

    match *rhs {
        Item::ConstInteger(m) => match *lhs {
            Item::ConstInteger(_) => return Err(ErrType::ParserNotFoldingConstants),

            Item::LocalVar(_) | Item::Add(_, _) | Item::Sub(_, _) => {
                listing.extend_from_slice(&cg_item(*lhs, st, DataDest::RegA, CtrlDest::Next, rc_a)?);
                if m == 1 {
                    decrement_a(&mut listing, rc_a);
                } else if m == !0 {
                    increment_a(&mut listing, rc_a);
                } else {
                    subtract_a_imm16(&mut listing, rc_a, m);
                }
                listing.extend_from_slice(&cg_store_a(dd, rc_a)?);
                listing.extend_from_slice(&cg_goto(cd)?);
            }

            _ => return Err(ErrType::UnexpectedCGArgs),
        }

        Item::LocalVar(ofs) => match *lhs {
            Item::ConstInteger(_) | Item::LocalVar(_) | Item::Add(_, _) | Item::Sub(_, _) => {
                listing.extend_from_slice(&cg_item(*lhs, st, DataDest::RegA, CtrlDest::Next, rc_a)?);
                subtract_a_local(&mut listing, rc_a, ofs);
                listing.extend_from_slice(&cg_store_a(dd, rc_a)?);
                listing.extend_from_slice(&cg_goto(cd)?);
            }

            _ => return Err(ErrType::UnexpectedCGArgs),
        }

        Item::Add(_, _) | Item::Sub(_, _) => match *lhs {
            Item::ConstInteger(_) | Item::LocalVar(_) | Item::Add(_, _) | Item::Sub(_, _) => {
                let t: TargetByte = st.alloc_temp() as TargetByte;
                listing.extend_from_slice(&cg_item(*lhs, st, DataDest::Local(t), CtrlDest::Next, rc_a)?);
                listing.extend_from_slice(&cg_item(*rhs, st, DataDest::RegA, CtrlDest::Next, rc_a)?);
                subtract_a_local(&mut listing, rc_a, t);
                listing.extend_from_slice(&cg_store_a(dd, rc_a)?);
                listing.extend_from_slice(&cg_goto(cd)?);
                st.free_temp();
            }

            _ => return Err(ErrType::UnexpectedCGArgs),
        }

        _ => return Err(ErrType::UnexpectedCGArgs),
    }

    Ok(listing)
}

pub fn cg_assignment(lhs: Item, rhs: Item, st: &mut SymTab, dd: DataDest, cd: CtrlDest, rc_a: &mut RegCache) -> Result<Vec<Ins>, ErrType> {
    if (dd != DataDest::RegA) && (dd != DataDest::Effect) {
        return Err(ErrType::UnexpectedCGArgs);
    }

    match lhs {
        Item::LocalVar(offset) => {
            let mut listing: Vec<Ins> = Vec::new();

            listing.extend_from_slice(&cg_item(rhs, st, DataDest::RegA, CtrlDest::Next, rc_a)?);
            store_a_local(&mut listing, rc_a, offset);
            listing.extend_from_slice(&cg_goto(cd)?);

            Ok(listing)
        }

        _ => Err(ErrType::LValExpected),
    }
}

pub fn cg_item(item: Item, st: &mut SymTab, dd: DataDest, cd: CtrlDest, rc_a: &mut RegCache) -> Result<Vec<Ins>, ErrType> {
    match item {
        Item::DeclareLocal(id, rval) => cg_declare_local(st, id, *rval, dd, cd, rc_a),
        Item::ConstInteger(n) => cg_const_int(n, dd, cd, rc_a),
        Item::LocalVar(offset) => cg_local(offset, dd, cd, rc_a),
        Item::StatementList(statements) => cg_statement_list(statements, st, dd, cd, rc_a),
        Item::Add(lhs, rhs) => cg_add(lhs, rhs, st, dd, cd, rc_a),
        Item::Sub(lhs, rhs) => cg_sub(lhs, rhs, st, dd, cd, rc_a),
        Item::Assign(lhs, rhs) => cg_assignment(*lhs, *rhs, st, dd, cd, rc_a),

        _ => Err(ErrType::ExpressionExpected),
    }
}

pub fn cg_goto(cd: CtrlDest) -> Result<Vec<Ins>, ErrType> {
    match cd {
        CtrlDest::Next => Ok(vec![]),               // just drop through to next instruction
        CtrlDest::Return => Ok(vec![Ins::Return]),  // return from current subroutine
    }
}

pub fn cg_store_a(dd: DataDest, rc_a: &mut RegCache) -> Result<Vec<Ins>, ErrType> {
    let mut listing: Vec<Ins> = Vec::new();

    match dd {
        DataDest::RegA => (),   // Already there.
        DataDest::Effect => (), // Already done.
        DataDest::Local(ofs) => store_a_local(&mut listing, rc_a, ofs),
    }

    Ok(listing)
}
