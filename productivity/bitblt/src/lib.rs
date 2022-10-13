/// Indicates how to combine the source data and destination data
/// when performing a bitblt operation.  There are sixteen operations
/// to select from.
///
/// Black will always fill destination bits with zeros.
/// White will always fill destination bits with ones.
#[repr(usize)]
#[derive(Debug)]
pub enum BlitOp {
    Black,
    Nor,
    NotDandS,
    NotD,
    DandNotS,
    NotS,
    Xor,
    Nand,
    And,
    Xnor,
    S,
    SorNotD,
    D,
    DorNotS,
    Or,
    White,
}

/// Encapsulates the current state of a bitblt operation.  This structure should be initialized
/// according to the type of blit operation you wish to perform prior to invoking any other bitblt
/// functions.
pub struct BlitContext<'a> {
    //--- Byte-wise Fields

    /// Reference to a slice of bytes containing source data.
    pub s_bits: &'a [u8],

    /// Index of the next byte to be read from `s_bits`.
    pub s_ptr: usize,

    /// The number of bits to shift data read from `s_bits`.
    pub s_shift: u8,

    /// After reading the next byte from `s_bits` and
    /// shifting by `s_shift` bits, this mask is logically-ANDed
    /// to yield the final source value.
    pub s_mask: u8,

    /// The previous byte read from `s_bits`.  
    pub s_data: u8,

    /// Mutable reference to a slice of bytes containing the
    /// destination data.
    pub d_bits: &'a mut [u8],

    /// Index of the next byte to be written to `d_bits`.
    pub d_ptr: usize,

    /// The logical operation to apply to the source and destination data.
    pub operation: BlitOp,
}

/// Given a source and a destination bitmap,
/// compute a new destination byte from
/// a single source byte and a destination byte.
///
/// This function encapsulates the core of the bit-blit algorithm.
/// It works as follows:
///
/// Read a byte from the source channel (`s_bits`);
/// shift it to the right by `s_shift` bits and
/// merge with the discarded bits of the previously read byte.
/// Then, apply the mask in `s_mask`.
///
/// Next, read a byte from the destination channel (`d_bits`),
/// and combine with the shifted and masked byte from above
/// according to the selected operation (see [[BlitOp]]).
/// Write the combined value back to the destination.
///
/// Increment the source and destination pointers.
pub fn blit_byte_ascending(bc: &mut BlitContext) {
    let raw_s = bc.s_bits[bc.s_ptr];
    let s = bc.s_mask
        & ((raw_s as u16 >> bc.s_shift) | ((bc.s_data as u16) << (8 - bc.s_shift))) as u8;
    let d = bc.d_bits[bc.d_ptr];

    let d = match bc.operation {
        BlitOp::Black => 0u8,
        BlitOp::Nor => !(s | d),
        BlitOp::NotDandS => !d & s,
        BlitOp::NotD => !d,
        BlitOp::DandNotS => d & !s,
        BlitOp::NotS => !s,
        BlitOp::Xor => s ^ d,
        BlitOp::Nand => !(s & d),
        BlitOp::And => s & d,
        BlitOp::Xnor => !(s | d),
        BlitOp::S => s,
        BlitOp::SorNotD => s | !d,
        BlitOp::D => d,
        BlitOp::DorNotS => d | !s,
        BlitOp::Or => s | d,
        BlitOp::White => 0xFFu8,
    };

    bc.d_bits[bc.d_ptr] = d;
    bc.d_ptr = bc.d_ptr.overflowing_add(1).0;
    bc.s_ptr = bc.s_ptr.overflowing_add(1).0;
    bc.s_data = raw_s;
}

/// Given a source and a destination bitmap,
/// compute a new destination byte from
/// a single source byte and a destination byte.
///
/// This function encapsulates the core of the bit-blit algorithm.
/// It works as follows:
///
/// Read a byte from the source channel (`s_bits`);
/// shift it to the left by `s_shift` bits and
/// merge with the discarded bits of the previously read byte.
/// Then, apply the mask in `s_mask`.
///
/// Next, read a byte from the destination channel (`d_bits`),
/// and combine with the shifted and masked byte from above
/// according to the selected operation (see [[BlitOp]]).
/// Write the combined value back to the destination.
///
/// Decrement the source and destination pointers.
pub fn blit_byte_descending(bc: &mut BlitContext) {
    let raw_s = bc.s_bits[bc.s_ptr];
    let s = bc.s_mask
        & (((raw_s as u16) << bc.s_shift) | ((bc.s_data as u16) >> (8 - bc.s_shift))) as u8;
    let d = bc.d_bits[bc.d_ptr];

    let d = match bc.operation {
        BlitOp::Black => 0u8,
        BlitOp::Nor => !(s | d),
        BlitOp::NotDandS => !d & s,
        BlitOp::NotD => !d,
        BlitOp::DandNotS => d & !s,
        BlitOp::NotS => !s,
        BlitOp::Xor => s ^ d,
        BlitOp::Nand => !(s & d),
        BlitOp::And => s & d,
        BlitOp::Xnor => !(s | d),
        BlitOp::S => s,
        BlitOp::SorNotD => s | !d,
        BlitOp::D => d,
        BlitOp::DorNotS => d | !s,
        BlitOp::Or => s | d,
        BlitOp::White => 0xFFu8,
    };

    bc.d_bits[bc.d_ptr] = d;
    bc.d_ptr = bc.d_ptr.overflowing_sub(1).0;
    bc.s_ptr = bc.s_ptr.overflowing_sub(1).0;
    bc.s_data = raw_s;
}

#[cfg(test)]
mod tests;
