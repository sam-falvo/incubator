extern crate sdlstate;
extern crate stencil;
extern crate sdl2;

use bitblt::{BlitOp, BlitContext, blit_rect};
use stencil::stencil::{Stencil, Draw, Pattern};
use stencil::types::{Dimension, Unit, Point, Rect};
use stencil::simple_bitmap_font::SimpleBitmapFont;

use sdlstate::SdlState;
use std::{thread, time};

use sdl2::event::{Event, WindowEvent};

const W: Dimension = 320;
const H: Dimension = 200;

static DESKTOP_PATTERN: Pattern = [0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55];
static BLACK_PATTERN: Pattern = [0, 0, 0, 0, 0, 0, 0, 0];
static WHITE_PATTERN: Pattern = [255, 255, 255, 255, 255, 255, 255, 255];

fn draw_dialog_box(
    st: &mut impl Draw,
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

fn repaint(desktop: &mut Stencil, r: Rect, sdl: &mut SdlState) {
    let ((left, top), (right, bottom)) = r;
    let (left, top) = (left as usize, top as usize);
    let (right, bottom) = (right as usize, bottom as usize);
    let width = right - left;
    let height = bottom - top;

    sdl.paint_with(|ctx| {
        ctx.paste_stamp_be(
            (left, top),
            (width, height),
            desktop.get_span(),
            (left, top),
            desktop.borrow_bits(),
        );
    });
}

fn main() {
    let mut sdl = SdlState::new("Clock Demo", W as u32, H as u32);
    let mut event_pump = sdl.context.event_pump().unwrap();
    let mut event_iter = event_pump.wait_iter();

    let mut desktop = Stencil::new_with_dimensions(W, H);

    let mut done = false;
    let mut command = demo_init(&mut desktop);
    while !done {
        match command {
            Cmd::Nop => (),
            Cmd::Quit => done = true,
            Cmd::Repaint(r) => repaint(&mut desktop, r, &mut sdl),
            Cmd::WaitEvent => {
                let event = event_iter.next();

                command = Cmd::Nop;
                if let Some(e) = event {
                    match e {
                        Event::Quit {..} => command = Cmd::Quit,
                        Event::Window {timestamp: _, window_id: _, win_event: we} => {
                            if we == WindowEvent::Exposed {
                                repaint(&mut desktop, ((0, 0), (W, H)), &mut sdl)
                            }
                        }
                        _ => ()
                    }
                }
            },
        };
        command = demo_tick(&mut desktop, command);
    }
}

use crate::stencil::sysfont_bsw_9::SYSTEM_BITMAP_FONT;

enum Cmd {
    Nop,
    Quit,
    Repaint(Rect),
    WaitEvent,
}

static CLOSE_BITMAP: [u8; 30] = [
    0b11111111, 0b11110000, 0,
    0b11000000, 0b00110000, 0,
    0b11011001, 0b10110000, 0,
    0b11001111, 0b00110000, 0,
    0b11000110, 0b00110000, 0,
    0b11000110, 0b00110000, 0,
    0b11001111, 0b00110000, 0,
    0b11011001, 0b10110000, 0,
    0b11000000, 0b00110000, 0,
    0b11111111, 0b11110000, 0,
];

fn demo_init(desktop: &mut Stencil) -> Cmd {
    let (w, h) = desktop.dimensions;

    desktop.filled_rectangle((0, 0), (w, h), &DESKTOP_PATTERN);
    draw_dialog_box(desktop, 80, 50, 240, 150);
    {
        // to scope a mutable borrow
        let mut bc = BlitContext::new(&CLOSE_BITMAP, 3, &mut desktop.bits, (w >> 3) as usize);
        blit_rect(&mut bc, 0, 0, 12, 10, 82, 51, BlitOp::DandNotS);
    }
    desktop.horizontal_line((80, 62), 240, 0x00);

    let font = SYSTEM_BITMAP_FONT;
    let op = BlitOp::Xor;
    let _ = paint_text(desktop, op, &font, 98, 52 + font.baseline, "<-- Click to close").unwrap();

    Cmd::Repaint(((0, 0), (w, h)))
}

fn demo_tick(desktop: &mut Stencil, previous: Cmd) -> Cmd {
    match previous {
        Cmd::Quit => previous,
        _ => Cmd::WaitEvent,
    }
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
