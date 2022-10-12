extern crate sdlstate;
extern crate stencil;

use stencil::stencil::Stencil;
use stencil::types::{Dimension, Unit};

use sdlstate::SdlState;
use std::{thread, time};

const W: Dimension = 320;
const H: Dimension = 200;

static DESKTOP_PATTERN: [u8; 8] = [0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55];

static BLACK_PATTERN: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];

static WHITE_PATTERN: [u8; 8] = [255, 255, 255, 255, 255, 255, 255, 255];

fn draw_dialog_box(
    st: &mut Stencil,
    paper_left: Unit,
    paper_top: Unit,
    paper_right: Unit,
    paper_bottom: Unit,
) {
    let border_left = paper_left - 2;
    let border_top = paper_top - 1;
    let border_right = paper_right + 2;
    let border_bottom = paper_bottom + 1;

    let shadow_left = border_left + 2;
    let shadow_top = border_top + 1;
    let shadow_right = border_right + 2;
    let shadow_bottom = border_bottom + 1;

    st.filled_rectangle(
        (shadow_left, shadow_top),
        (shadow_right, shadow_bottom),
        &BLACK_PATTERN,
    );
    st.filled_rectangle(
        (border_left, border_top),
        (border_right, border_bottom),
        &BLACK_PATTERN,
    );
    st.filled_rectangle(
        (paper_left, paper_top),
        (paper_right, paper_bottom),
        &WHITE_PATTERN,
    );
}

fn main() {
    let mut sdl = SdlState::new("Clock Demo", W as u32, H as u32);
    let desktop_opt = Stencil::new_with_dimensions(W as usize, H as usize);

    match desktop_opt {
        Some(mut desktop) => {
            desktop.filled_rectangle((0, 0), (W, H), &DESKTOP_PATTERN);
            for i in 0..65536 {
                draw_dialog_box(&mut desktop, 80, 50, 240, 150);

                demo(&mut desktop, i);

                sdl.paint_with(|ctx| {
                    ctx.paste_stamp_be(
                        (0, 0),
                        (W as usize, H as usize),
                        desktop.get_span(),
                        (0, 0),
                        desktop.borrow_bits(),
                    );
                });

                thread::sleep(time::Duration::new(0, 100000000));
            }
        }

        _ => {
            panic!("Thou dost expect too much.");
        }
    };
}

use crate::stencil::sysfont_bsw_9::SYSTEM_BITMAP_FONT;

#[derive(Debug)]
pub struct BlitContext<'a> {
    // Individual column controls
    pub src_bits: &'a [u8],
    pub src_ptr: usize,
    pub src_data: u8,
    pub src_shift: u8,
    pub src_mask: u8,

    pub dst_bits: &'a mut [u8],
    pub dst_ptr: usize,

    pub operation: BlitOp,

    // Line controls
    pub src_fc_mask: u8,
    pub src_lc_mask: u8,
    pub src_span: usize,
    pub dst_span: usize,
}

impl<'a> BlitContext<'a> {
    pub fn new(src_bits: &'a [u8], src_span: usize, dst_bits: &'a mut [u8], dst_span: usize, op: BlitOp) -> Self {
        BlitContext {
            src_bits,
            src_ptr: 0,
            src_data: 0,
            src_shift: 0,
            src_mask: 0xFF,

            dst_bits,
            dst_ptr: 0,

            operation: op,

            src_fc_mask: 0,
            src_lc_mask: 0,
            src_span,
            dst_span,
        }
    }
}

// 0bABCD
//   ||||
//   |||`-- !D & !S
//   ||`--- !D &  S
//   |`----  D & !S
//   `-----  D &  S
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

static DUMMY_BITS: [u8; 8] = [
    0b00000001,
    0b00000001,
    0b00000001,
    0b00000001,
    0b00000001,
    0b00000001,
    0b00000001,
    0b00000001,
];

fn demo(desktop: &mut Stencil, frame: usize) {
    let font = SYSTEM_BITMAP_FONT;
    let desktop_span = ((desktop.dimensions.0 + 7) >> 3) as usize;
    let mut bc = BlitContext::new(font.bits, font.span, &mut desktop.bits, desktop_span, BlitOp::DandNotS);
//    let mut bc = BlitContext::new(&DUMMY_BITS, 1, &mut desktop.bits, desktop_span, BlitOp::DandNotS);
    let x = (frame & 31) as Unit;
    let left_to_right = (frame & 32) != 0;

    let leftA = font.left_edges[65-32] as Unit;
    let rightA = font.left_edges[66-32] as Unit;

    if left_to_right {
        blit_rect(&mut bc, leftA, 0, rightA, 9, 160 + x, 100);
    } else {
        blit_rect(&mut bc, leftA, 0, rightA, 9, 160 - x, 100);
    }
}

static RIGHT_COLUMN_MASKS: [u8; 8] = [
    0b10000000, 0b11000000, 0b11100000, 0b11110000, 0b11111000, 0b11111100, 0b11111110,
    0b11111111,
];

static LEFT_COLUMN_MASKS: [u8; 8] = [
    0b11111111, 0b01111111, 0b00111111, 0b00011111, 0b00001111, 0b00000111, 0b00000011,
    0b00000001,
];

fn blit_rect(
    bc: &mut BlitContext,
    src_left: Unit,
    src_top: Unit,
    src_right: Unit,
    src_bottom: Unit,
    dst_left: Unit,
    dst_top: Unit,
) {
    let inclusive_src_right = src_right - 1;

    let src_left_column = src_left >> 3;
    let src_right_column = inclusive_src_right >> 3;
    let src_left_pix = src_left & 7;

    let width_in_pixels = src_right - src_left;
    let height = src_bottom - src_top;

    let dst_right = dst_left + width_in_pixels;
    let inclusive_dst_right = dst_right - 1;
    let dst_bottom = dst_top + height;
    let dst_left_column = dst_left >> 3;
    let dst_right_column = inclusive_dst_right >> 3;
    let dst_left_pix = dst_left & 7;

    let src_width_in_columns = src_right_column - src_left_column + 1;
    let dst_width_in_columns = dst_right_column - dst_left_column + 1;
    let width_in_columns = src_width_in_columns.max(dst_width_in_columns);

    let ascending_mode = src_left_pix < dst_left_pix;
    let src_shift = (dst_left_pix - src_left_pix).abs();

    let mut src_fc_mask: u8;
    let mut src_lc_mask: u8;
    let mut src_ptr = (bc.src_span * src_top as usize) + src_left_column as usize;
    let mut dst_ptr = (bc.dst_span * dst_top as usize) + dst_left_column as usize;

    if ascending_mode {
        src_fc_mask = LEFT_COLUMN_MASKS[(dst_left & 7) as usize];
        src_lc_mask = RIGHT_COLUMN_MASKS[(inclusive_dst_right & 7) as usize];
    } else {
        src_fc_mask = RIGHT_COLUMN_MASKS[(inclusive_dst_right & 7) as usize];
        src_lc_mask = LEFT_COLUMN_MASKS[(dst_left & 7) as usize];

        src_ptr += (bc.src_span * (height - 1) as usize) + (width_in_columns - 1) as usize;
        dst_ptr += (bc.dst_span * (height - 1) as usize) + (width_in_columns - 1) as usize;
    }

    if dst_width_in_columns == 1 {
        src_lc_mask &= src_fc_mask;
        src_fc_mask = 0;
    } else if dst_width_in_columns < src_width_in_columns {
        src_fc_mask = 0;
    }

    bc.src_shift = src_shift as u8;
    bc.src_fc_mask = src_fc_mask;
    bc.src_lc_mask = src_lc_mask;
    bc.dst_ptr = dst_ptr;
    bc.src_ptr = src_ptr;

    if ascending_mode {
        for _ in 0..height {
            blit_line_ascending(bc, width_in_columns as usize);
        }
    } else {
        for _ in 0..height {
            blit_line_descending(bc, width_in_columns as usize);
        }
    }
}

fn blit_line_ascending(bc: &mut BlitContext, width: usize) {
    bc.src_data = 0;
    match width {
        0 => (),
        1 => {
            bc.src_mask = bc.src_fc_mask & bc.src_lc_mask;
            blit_byte_ascending(bc);
        }
        _ => {
            let mut whats_left = width - 1;
            bc.src_mask = bc.src_fc_mask;
            blit_byte_ascending(bc);

            bc.src_mask = 0xFF;
            while whats_left > 1 {
                blit_byte_ascending(bc);
                whats_left -= 1;
            }

            bc.src_mask = bc.src_lc_mask;
            blit_byte_ascending(bc);
        }
    }
    bc.src_ptr = bc.src_ptr.overflowing_add(bc.src_span - width).0;
    bc.dst_ptr = bc.dst_ptr.overflowing_add(bc.dst_span - width).0;
}

fn blit_line_descending(bc: &mut BlitContext, width: usize) {
    bc.src_data = 0;
    match width {
        0 => (),
        1 => {
            bc.src_mask = bc.src_fc_mask & bc.src_lc_mask;
            blit_byte_descending(bc);
        }
        _ => {
            let mut whats_left = width - 1;
            bc.src_mask = bc.src_fc_mask;
            blit_byte_descending(bc);

            bc.src_mask = 0xFF;
            while whats_left > 1 {
                blit_byte_descending(bc);
                whats_left -= 1;
            }

            bc.src_mask = bc.src_lc_mask;
            blit_byte_descending(bc);
        }
    }
    bc.src_ptr = bc.src_ptr.overflowing_sub(bc.src_span - width).0;
    bc.dst_ptr = bc.dst_ptr.overflowing_sub(bc.dst_span - width).0;
}

fn blit_byte_ascending(bc: &mut BlitContext) {
    let raw_s = bc.src_bits[bc.src_ptr];
    let s = bc.src_mask
        & ((raw_s as u16 >> bc.src_shift) | ((bc.src_data as u16) << (8 - bc.src_shift))) as u8;
    let d = bc.dst_bits[bc.dst_ptr];

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

    bc.dst_bits[bc.dst_ptr] = d;
    bc.dst_ptr = bc.dst_ptr.overflowing_add(1).0;
    bc.src_ptr = bc.src_ptr.overflowing_add(1).0;
    bc.src_data = raw_s;
}

fn blit_byte_descending(bc: &mut BlitContext) {
    let raw_s = bc.src_bits[bc.src_ptr];
    let s = bc.src_mask
        & (((raw_s as u16) << bc.src_shift) | ((bc.src_data as u16) >> (8 - bc.src_shift))) as u8;
    let d = bc.dst_bits[bc.dst_ptr];

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

    bc.dst_bits[bc.dst_ptr] = d;
    bc.dst_ptr = bc.dst_ptr.overflowing_sub(1).0;
    bc.src_ptr = bc.src_ptr.overflowing_sub(1).0;
    bc.src_data = raw_s;
}
