use crate::text::paint_text;
use crate::Cmd;
use bitblt::{blit_rect, BlitContext, BlitOp};
use chrono::prelude::{DateTime, Local};
use stencil::stencil::{Draw, Pattern, Stencil};
use stencil::sysfont_bsw_9::SYSTEM_BITMAP_FONT;
use stencil::types::{Point, Unit};

static DESKTOP_PATTERN: Pattern = [0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55];
static BLACK_PATTERN: Pattern = [0, 0, 0, 0, 0, 0, 0, 0];
static WHITE_PATTERN: Pattern = [255, 255, 255, 255, 255, 255, 255, 255];

/// A monochrome bitmap of the application close gadget.
/// Although only 12x10 pixels, it's arranged as a 24x10 bitmap
/// to accomodate a bitblit::blit_rect bug.
/// See its docs and unit tests for more details.
static CLOSE_BITMAP: [u8; 30] = [
    0b11111111, 0b11110000, 0,
    0b10000000, 0b00010000, 0,
    0b10111111, 0b11010000, 0,
    0b10101010, 0b11010000, 0,
    0b10110101, 0b01010000, 0,
    0b10101010, 0b11010000, 0,
    0b10110101, 0b01010000, 0,
    0b10111111, 0b11010000, 0,
    0b10000000, 0b00010000, 0,
    0b11111111, 0b11110000, 0,
];

/// Draws a simple dialog box onto the provided stencil.
fn draw_dialog_box(
    st: &mut Stencil,
    paper_left: Unit,
    paper_top: Unit,
    paper_right: Unit,
    paper_bottom: Unit,
) {
    let border_left = paper_left - 1;
    let border_top = paper_top - 1;
    let border_right = paper_right + 1;
    let border_bottom = paper_bottom + 1;

    let shadow_left = border_left + 1;
    let shadow_top = border_top + 1;
    let shadow_right = border_right + 1;
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

/// Configure the initial state of the clock application.
pub fn demo_init(desktop: &mut Stencil) -> Cmd {
    let (w, h) = desktop.dimensions;

    desktop.filled_rectangle((0, 0), (w, h), &DESKTOP_PATTERN);
    draw_dialog_box(desktop, 80, 50, 240, 150);
    {
        // to scope a mutable borrow
        let mut bc = BlitContext::new(&CLOSE_BITMAP, 3, &mut desktop.bits, (w >> 3) as usize);
        blit_rect(&mut bc, 0, 0, 12, 10, 81, 51, BlitOp::DandNotS);
    }
    desktop.horizontal_line((80, 62), 240, 0x00);

    let font = SYSTEM_BITMAP_FONT;
    let op = BlitOp::Xor;
    let _ = paint_text(
        desktop,
        op,
        &font,
        98,
        52 + font.baseline,
        "<-- Click to close",
    )
    .unwrap();

    Cmd::Repaint(((0, 0), (w, h)))
}

/// Process input events for the clock application.
pub fn demo_tick(desktop: &mut Stencil, previous: Cmd) -> Cmd {
    match previous {
        Cmd::Quit => previous,
        Cmd::ButtonUp { at: point, .. } => {
            if clicked_in_close_gadget(point) {
                Cmd::Quit
            } else {
                Cmd::WaitEvent
            }
        }
        Cmd::TimerTick => redraw_time(desktop),
        _ => Cmd::WaitEvent,
    }
}

fn clicked_in_close_gadget(point: Point) -> bool {
    let (x, y) = point;
    (81 <= x) && (x < 94) && (50 <= y) && (y < 62)
}

static mut HIDE_COLON: bool = false;

/// Redraw the current time
fn redraw_time(desktop: &mut Stencil) -> Cmd {
    let dt: DateTime<Local> = Local::now();
    let time_string_1 = format!("{}", dt.format("%H:%M"));
    let time_string_2 = format!("{}", dt.format(":%S"));
    let font = &SYSTEM_BITMAP_FONT;
    let y = 100 + font.baseline;

    // Erase the old time pattern.  70px x 20px ought to be enough for anyone.

    desktop.filled_rectangle((160, 100), (230, 120), &WHITE_PATTERN);

    // Print the time string, but do so in a way that lets us capture where the seconds colon sits
    // on the screen.

    let xcolon = paint_text(
        desktop,
        BlitOp::Xor,
        font,
        160,
        y,
        &time_string_1,
    ).unwrap();

    let xmax = paint_text(
        desktop,
        BlitOp::Xor,
        font,
        xcolon,
        y,
        &time_string_2,
    ).unwrap();

    // Now, if we need to hide the colon, use the XOR operator to remove it.

    unsafe {
        HIDE_COLON = !HIDE_COLON;
        if HIDE_COLON {
            let _ = paint_text(
                desktop,
                BlitOp::Xor,
                font,
                xcolon,
                y,
                ":"
            ).unwrap();
        }
    }

    // Tell event loop to commit changes to the desktop.

    Cmd::Repaint(((160, 100), (xmax, 100 + font.height)))
}
