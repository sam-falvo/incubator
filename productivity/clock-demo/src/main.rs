extern crate sdlstate;
extern crate stencil;

use bitblt::{BlitOp, BlitContext, blit_rect};
use stencil::stencil::Stencil;
use stencil::types::{Dimension, Unit};
use stencil::simple_bitmap_font::SimpleBitmapFont;

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

fn demo(desktop: &mut Stencil, _frame: usize) {
    let font = SYSTEM_BITMAP_FONT;
    let op = BlitOp::Xor;

    let _ = paint_text(desktop, op, &font, 84, 60, "Hello world!").unwrap();
}

fn paint_text(stencil: &mut Stencil, op: BlitOp, font: &SimpleBitmapFont, mut x: Unit, y: Unit, text: &str) -> Option<Unit> {
    for b in text.bytes() {
        let new_x_opt = paint_char(stencil, op, font, x, y, b as u8);
        if let Some(new_x) = new_x_opt {
            x = new_x;
        } else {
            return None;
        }
    }
    Some(x)
}

// Try to paint a character onto the stencil.  If not able to do so, answer with None.  Otherwise,
// an updated X-coordinate is returned.
fn paint_char(stencil: &mut Stencil, op: BlitOp, font: &SimpleBitmapFont, x: Unit, y: Unit, mut char: u8) -> Option<Unit> {
    // If not representable in the glyph set of the font, assume the undefined character glyph,
    // which by definition, is always at highest_char+1 mod 256.
    let highest_character = font.highest_char;
    if (char < font.lowest_char) || (char > highest_character) {
        char = highest_character.overflowing_add(1).0;
    }
    char -= font.lowest_char;

    // Let's expand this to a valid array index.
    let char = char as usize;

    let left_edge = font.left_edges[char];
    let right_edge = font.left_edges[char + 1];
    let glyph_width = right_edge - left_edge;

    // Check to see if we can print this character on the stencil.
    // If not, return None.
    let stencil_width = stencil.dimensions.0;
    let new_cursor_position = x + glyph_width as Unit;
    if new_cursor_position > stencil_width {
        return None;
    }

    // Blit the character glyph onto the stencil.
    //
    // Because of a bug in bitblt that I do not know how to resolve yet, we must do this in two
    // steps.  First, we must blit the glyph into a temporary holding stencil that is sized large
    // enough for the glyph, but which holds the *pre-shifted* version of the glyph.

    let x_in_byte = (x & 7) as usize;

    let mut tmp_buf: [u8; 48] = [0; 48]; // Big enough for fonts up to 16 pixels tall, and with characters no wider than 16px.
    let mut bc = BlitContext::new(font.bits, font.span, &mut tmp_buf, 3);
    blit_rect(&mut bc, left_edge as usize, 0, right_edge as usize, font.height as usize, x_in_byte, 0, BlitOp::S);


    // Next, blit the pre-shifted glyph onto the destination bitmap with the operation requested by
    // the caller.  This requires resetting the BlitContext to refer to the correct stencil
    // bitmaps.
    bc.s_bits = &(*bc.d_bits);
    bc.s_span = 3;

    bc.d_bits = &mut stencil.bits;
    bc.d_span = ((stencil_width + 7) >> 3) as usize;

    let top = (y - font.baseline) as usize;
    blit_rect(&mut bc, x_in_byte, 0, x_in_byte + glyph_width as usize, font.height as usize, x as usize, top, op);

    // Return the next glyph location.
    Some(new_cursor_position)
}

