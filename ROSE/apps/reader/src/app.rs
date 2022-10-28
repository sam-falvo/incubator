use std::fs;
use std::cell::RefCell;

use stencil::stencil::{Draw, Stencil};
use stencil::utils::{ draw_desktop, draw_dialog_box };
use stencil::simple_bitmap_font::SimpleBitmapFont;
use stencil::sysfont_bsw_9::SYSTEM_BITMAP_FONT;
use stencil::simple_printer::SimplePrinter;
use stencil::types::Rect;
use stencil::utils::{WHITE_PATTERN, LINE_BLACK};

pub struct Reader {
    text_to_view: RefCell<Option<String>>,
}

impl Reader {
    pub fn new() -> Self {
        Reader{
            text_to_view: RefCell::new(None),
        }
    }

    pub fn print_file(&self, desktop: &mut Stencil, font: &SimpleBitmapFont) {
        let maybe_contents = self.text_to_view.replace(None);

        match maybe_contents {
            Some(text) => {
                let (width, height) = desktop.get_dimensions();
                let mut printer = SimplePrinter::new(desktop, ((10, 10), (width - 24, height - 10)), &font);
                printer.print(&text);
                self.text_to_view.replace(Some(text));
            },

            _ => (),
        }
    }
}

pub trait Initializable {
    fn init(&mut self, desktop: &mut Stencil);
}

impl Initializable for Reader {
    fn init(&mut self, desktop: &mut Stencil) {
        let (width, height) = desktop.get_dimensions();
        let font = SYSTEM_BITMAP_FONT;

        let (left, top) = (8, 8);
        let (right, bottom) = (width - 8, height - 8);
        let topleft = (left, top);
        let bottomright = (right, bottom);

        let text_right = right - 18;
        let prop_left = text_right + 2;
        let prop_top = top;
        let prop_right = right;
        let button_height = 16;
        let prop_bottom = bottom - 2*button_height;

        draw_desktop(desktop);
        draw_dialog_box(desktop, (topleft, bottomright));

        let file_contents = fs::read_to_string("lorem-ipsum.txt");
        match file_contents {
            Err(e) => {
                let error_reason = format!("Could not open lorem-ipsum.txt because...\n{}", e);
                let mut printer = SimplePrinter::new(desktop, ((10, 10), (text_right, height - 10)), &font);
                printer.print(&error_reason);
            },
            Ok(contents) => {
                self.text_to_view.replace(Some(contents));
                self.print_file(desktop, &font);
            }
        }

        let icon_y = 4;
        let up_top = prop_bottom;
        let up_bottom = up_top + button_height;
        let dn_top = up_bottom;
        let dn_bottom = dn_top + button_height;

        use bitblt::{BlitContext, BlitOp, blit_rect};
        draw_prop_gadget(desktop, ((prop_left, prop_top), (prop_right, prop_bottom)));
        desktop.framed_rectangle((prop_left, up_top), (prop_right, up_bottom), LINE_BLACK);
        desktop.framed_rectangle((prop_left, dn_top), (prop_right, dn_bottom), LINE_BLACK);
        {
            let mut bc = BlitContext::new(&ARROW_BITS_UP, 2, &mut desktop.bits, (width >> 3) as usize);
            blit_rect(&mut bc, 0, 0, 8, 8, (prop_left + 4) as usize, (up_top + icon_y) as usize, BlitOp::DandNotS);
        }
        {
            let mut bc = BlitContext::new(&ARROW_BITS_DN, 2, &mut desktop.bits, (width >> 3) as usize);
            blit_rect(&mut bc, 0, 0, 8, 8, (prop_left + 4) as usize, (dn_top + icon_y) as usize, BlitOp::DandNotS);
        }
    }
}

static ARROW_BITS_UP: [u8; 16] = [
    0b00011000, 0b00000000,
    0b00011000, 0b00000000,
    0b00111100, 0b00000000,
    0b00111100, 0b00000000,
    0b01111110, 0b00000000,
    0b00011000, 0b00000000,
    0b00011000, 0b00000000,
    0b00011000, 0b00000000,
];

static ARROW_BITS_DN: [u8; 16] = [
    0b00011000, 0b00000000,
    0b00011000, 0b00000000,
    0b00011000, 0b00000000,
    0b01111110, 0b00000000,
    0b00111100, 0b00000000,
    0b00111100, 0b00000000,
    0b00011000, 0b00000000,
    0b00011000, 0b00000000,
];

static SLIDER_PATTERN: [u8; 8] = [
    0b00010001,
    0b01000100,
    0b00010001,
    0b01000100,
    0b00010001,
    0b01000100,
    0b00010001,
    0b01000100,
];

fn draw_prop_gadget(desktop: &mut Stencil, area: Rect) {
    let slider_topleft = area.0;
    let slider_bottomright = area.1;
    let (slider_left, slider_top) = slider_topleft;
    let (slider_right, slider_bottom) = slider_bottomright;

    desktop.filled_rectangle(slider_topleft, slider_bottomright, &SLIDER_PATTERN);
    desktop.vertical_line(slider_topleft, slider_bottom, LINE_BLACK);

    let knob_left = slider_left + 2;
    let knob_right = slider_right - 1;
    let knob_top = slider_top + 2;
    let knob_bottom = (slider_top + slider_bottom) >> 1;
    let knob_topleft = (knob_left, knob_top);
    let knob_bottomright = (knob_right, knob_bottom);

    desktop.filled_rectangle(knob_topleft, knob_bottomright, &WHITE_PATTERN);
    desktop.framed_rectangle(knob_topleft, knob_bottomright, LINE_BLACK);
}

