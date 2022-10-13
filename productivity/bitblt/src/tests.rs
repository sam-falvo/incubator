use crate::{BlitOp, BlitContext};
use crate::{blit_byte_ascending, blit_byte_descending};

// Given that you have a properly initialized BlitContext structure,
// when you call blit_byte_ascending,
// then a single byte should be transferred, leaving the BlitContext in a state where subsequent
// calls to blit_byte_ascending will seamlessly continue the blit operation.
#[test]
fn blit_byte_ascending_tracks_source_bytes() {
    let src: [u8; 1] = [0x21];
    let mut dst: [u8; 1] = [0x00];

    let mut bc = BlitContext {
        s_bits: &src,
        s_ptr: 0,
        s_shift: 4,
        s_data: 0x45,
        s_mask: 0xFF,

        d_bits: &mut dst,
        d_ptr: 0,

        operation: BlitOp::Or,
    };

    blit_byte_ascending(&mut bc);

    assert_eq!(bc.d_bits[0], 0x52);
    assert_eq!(bc.s_data, 0x21);
    assert_eq!(bc.s_ptr, 1);
    assert_eq!(bc.d_ptr, 1);
}

// Given that you have a properly initialized BlitContext structure,
// when you call blit_byte_descending,
// then a single byte should be transferred, leaving the BlitContext in a state where subsequent
// calls to blit_byte_descending will seamlessly continue the blit operation.
#[test]
fn blit_byte_descending_tracks_source_bytes() {
    let src: [u8; 1] = [0x21];
    let mut dst: [u8; 1] = [0x00];

    let mut bc = BlitContext {
        s_bits: &src,
        s_ptr: 0,
        s_shift: 4,
        s_data: 0x45,
        s_mask: 0xFF,

        d_bits: &mut dst,
        d_ptr: 0,

        operation: BlitOp::Or,
    };

    blit_byte_descending(&mut bc);

    assert_eq!(bc.d_bits[0], 0x14);
    assert_eq!(bc.s_data, 0x21);
    assert_eq!(bc.s_ptr, !0);
    assert_eq!(bc.d_ptr, !0);
}

