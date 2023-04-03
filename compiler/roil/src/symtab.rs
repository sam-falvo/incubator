// vim:sw=4:ts=4:et:ai
//
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

    pub fn alloc_temp(&mut self) -> u16 {
        let next_local = self.next_local;
        self.next_local += 2;
        next_local
    }

    pub fn free_temp(&mut self) {
        self.next_local -= 2;
    }

    pub fn create_local(&mut self, name: &str) {
        self.names.push(name.to_string());
        let t = self.alloc_temp();
        self.offsets.push(t);
        self.length += 1;
    }

    pub fn find_by_name<'n>(&'n self, name: &'n str) -> Result<Box<Symbol>, Errors> {
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
        return Ok(Box::new(Symbol {
            name: name,
            offset: *self.offsets.get(i).ok_or(Errors::BadSymbolTableIndex)?,
        }));
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
