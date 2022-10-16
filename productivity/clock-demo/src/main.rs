extern crate sdlstate;
extern crate stencil;

use bitblt::{BlitOp, BlitContext, blit_rect};
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

                thread::sleep(time::Duration::new(0, 50000000));
            }
        }

        _ => {
            panic!("Thou dost expect too much.");
        }
    };
}

use crate::stencil::sysfont_bsw_9::SYSTEM_BITMAP_FONT;

fn demo(desktop: &mut Stencil, frame: usize) {
    let font = SYSTEM_BITMAP_FONT;
    let desktop_span = ((desktop.dimensions.0 + 7) >> 3) as usize;
    let mut bc = BlitContext::new(font.bits, font.span, &mut desktop.bits, desktop_span);
    let x = (frame & 31) as Unit;
    let left_to_right = (frame & 32) != 0;

    let left_a = font.left_edges[65-32] as Unit;
    let right_a = font.left_edges[66-32] as Unit;

    if left_to_right {
        blit_rect(&mut bc, left_a as usize, 0, right_a as usize, 9, (160 + x) as usize, 100, BlitOp::DandNotS);
    } else {
        blit_rect(&mut bc, left_a as usize, 0, right_a as usize, 9, (160 - x) as usize, 100, BlitOp::DandNotS);
    }
}

