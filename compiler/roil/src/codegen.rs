use crate::parser::{Item, TargetByte, TargetUInt};
use crate::symtab::SymTab;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Ins {
    LoadAImm16(TargetUInt),
    StoreADP(TargetByte),
    Return,
}

pub fn cg_item(item: Item, st: &mut SymTab) -> Vec<Ins> {
    match item {
        Item::DeclareLocal(ref id, rval) => {
            st.create_local(&id);
            match st.find_by_name(&id) {
                Ok(sym) => {
                    let offset = sym.offset as u8;
                    let mut sublisting = cg_item(*rval, st);
                    sublisting.push(Ins::StoreADP(offset));
                    return sublisting;
                }

                _ => return vec![],
            }
        }

        Item::ConstInteger(n) => vec![Ins::LoadAImm16(n)],

        // If we're here, we hit a syntax error that wasn't caught by the parser.
        // Return something sensible in this case.  TODO.
        _ => return vec![],
    }
}
