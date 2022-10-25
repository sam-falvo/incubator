use crate::Cmd;
use bitblt::{blit_rect, BlitContext, BlitOp};
use chrono::prelude::{DateTime, Local};
use stencil::printer::{Printer, SimplePrinter};
use stencil::stencil::{Draw, Stencil};
use stencil::sysfont_bsw_9::SYSTEM_BITMAP_FONT;
use stencil::types::{Point, Rect, Unit};
use stencil::utils::{draw_desktop, draw_dialog_box, WHITE_PATTERN};

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

/// Configure the initial state of the clock application.
pub fn demo_init(desktop: &mut Stencil) -> Cmd {
    let (w, h) = desktop.dimensions;

    draw_desktop(desktop);
    draw_dialog_box(desktop, ((80, 50), (240, 150)));
    {
        // to scope a mutable borrow
        let mut bc = BlitContext::new(&CLOSE_BITMAP, 3, &mut desktop.bits, (w >> 3) as usize);
        blit_rect(&mut bc, 0, 0, 12, 10, 81, 51, BlitOp::DandNotS);
    }
    desktop.horizontal_line((80, 62), 240, 0x00);

    {
        let font = &SYSTEM_BITMAP_FONT;
        let mut printer = SimplePrinter::new(desktop, ((96, 52), (238, 52 + font.height)), font);
        printer.print("<-- Click to close");
    }

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
    let label_region: Rect = ((160, 100), (230, 120));

    // Erase the old time pattern.
    desktop.filled_rectangle(label_region.0, label_region.1, &WHITE_PATTERN);

    // Print the time string, but do so in a way that lets us capture where the seconds colon sits
    // on the screen.
    let xcolon: Unit;
    let xmax: Unit;
    {
        let mut printer = SimplePrinter::new(desktop, label_region, &font);
        printer.print(&time_string_1);
        xcolon = printer.head.0;

        printer.print(&time_string_2);
        xmax = printer.head.0;
    }

    // Now, if we need to hide the colon, use the XOR operator to remove it.
    unsafe {
        HIDE_COLON = !HIDE_COLON;
        if HIDE_COLON {
            let mut printer = SimplePrinter::new(desktop, label_region, &font);
            printer.blit_op = BlitOp::Xor;
            printer.head.0 = xcolon;
            printer.print(":");
        }
    }

    // Tell event loop to commit changes to the desktop.

    Cmd::Repaint((label_region.0, (xmax, 100 + font.height)))
}
