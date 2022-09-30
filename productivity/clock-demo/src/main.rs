extern crate stencil;
extern crate sdlstate;

use stencil::Stencil;
use sdlstate::SdlState;
use std::{thread, time};

const W: usize = 320;
const H: usize = 200;

fn main() {
    let mut sdl = SdlState::new("Clock Demo", W as u32, H as u32);
    let mut strip = Stencil::new_with_dimensions(W, H);
    let desktop_pattern: [u8; 8] = [
        0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55,
    ];

    strip.filled_rectangle((0, 0), (W, H), &desktop_pattern);

    sdl.paint_with(|ctx| {
        ctx.paste_stamp_be((0, 0), (W, H), strip.span, (0, 0), strip.borrow_bits());
    });

    thread::sleep(time::Duration::new(15, 0));
}
