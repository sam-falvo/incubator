use crate::types::Unit;


pub struct SimpleBitmapFont<'a> {
    pub span: usize,
    pub height: Unit,
    pub baseline: Unit,
    pub bits: &'a [u8],
    pub left_edges: &'a [u16],
    pub lowest_char: u8,
    pub highest_char: u8,
}


