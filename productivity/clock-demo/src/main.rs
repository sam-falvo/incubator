extern crate stencil;
extern crate sdlstate;

use stencil::stencil::Stencil;
use stencil::types::{Unit, Dimension};

use sdlstate::SdlState;
use std::{thread, time};

const W: Dimension = 320;
const H: Dimension = 200;

static DESKTOP_PATTERN: [u8; 8] = [
    0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55,
];

static BLACK_PATTERN: [u8; 8] = [
    0, 0, 0, 0, 0, 0, 0, 0,
];

static WHITE_PATTERN: [u8; 8] = [
    255, 255, 255, 255, 255, 255, 255, 255,
];

fn draw_dialog_box(st: &mut Stencil, paper_left: Unit, paper_top: Unit, paper_right: Unit, paper_bottom: Unit) {
    let border_left = paper_left - 2;
    let border_top = paper_top - 1;
    let border_right = paper_right + 2;
    let border_bottom = paper_bottom + 1;

    let shadow_left = border_left + 2;
    let shadow_top = border_top + 1;
    let shadow_right = border_right + 2;
    let shadow_bottom = border_bottom + 1;

    st.filled_rectangle((shadow_left, shadow_top), (shadow_right, shadow_bottom), &BLACK_PATTERN);
    st.filled_rectangle((border_left, border_top), (border_right, border_bottom), &BLACK_PATTERN);
    st.filled_rectangle((paper_left, paper_top), (paper_right, paper_bottom), &WHITE_PATTERN);
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
                    ctx.paste_stamp_be((0, 0), (W as usize, H as usize), desktop.get_span(), (0, 0), desktop.borrow_bits());
                });

                thread::sleep(time::Duration::new(1, 16));
            }

            thread::sleep(time::Duration::new(15, 0));
        },

        _ => {
            panic!("Thou dost expect too much.");
        }
    };

}

use crate::stencil::sysfont_bsw_9::SYSTEM_BITMAP_FONT;

struct BlitContext<'a> {
    src_bits: &'a [u8],
    src_ptr: usize,
    src_data: u8,
    src_shift: u8,

    dst_bits: &'a mut [u8],
    dst_ptr: usize,

    operation: BlitOp,
}

// 0bABCD
//   ||||
//   |||`-- !D & !S
//   ||`--- !D &  S
//   |`----  D & !S
//   `-----  D &  S
#[repr(usize)]
enum BlitOp {
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

fn demo(desktop: &mut Stencil, frame: usize) {
    let font = SYSTEM_BITMAP_FONT;
    let mut bc = BlitContext {
        src_bits: font.bits,
        src_ptr: 0,
        src_data: 0,
        src_shift: (frame & 7) as u8,

        dst_bits: &mut desktop.bits,
        dst_ptr: 4016 + 720,

        operation: BlitOp::DandNotS,
    };

    for i in 0..9 {
        bc.src_data = 0;
        blit_byte_ascending(&mut bc);
        blit_byte_ascending(&mut bc);
        blit_byte_ascending(&mut bc);
        blit_byte_ascending(&mut bc);
        blit_byte_ascending(&mut bc);

        bc.src_ptr = bc.src_ptr.overflowing_add(60-5).0;
        bc.dst_ptr = bc.dst_ptr.overflowing_add(40-5).0;
    }
}

fn blit_byte_ascending(bc: &mut BlitContext) {
    let raw_s = bc.src_bits[bc.src_ptr];
    let s = ((raw_s as u16 >> bc.src_shift) | ((bc.src_data as u16) << (8 - bc.src_shift))) as u8;
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
    let s = (((raw_s as u16) << bc.src_shift) | ((bc.src_data as u16) >> (8 - bc.src_shift))) as u8;
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

