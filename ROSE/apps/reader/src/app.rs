use std::fs;
use std::cell::RefCell;

use stencil::stencil::{Draw, Stencil};
use stencil::utils::{ draw_desktop, draw_dialog_box };
use stencil::simple_bitmap_font::SimpleBitmapFont;
use stencil::sysfont_bsw_9::SYSTEM_BITMAP_FONT;
use stencil::simple_printer::SimplePrinter;
use stencil::types::Rect;
use stencil::utils::{WHITE_PATTERN, BLACK_PATTERN, LINE_BLACK};

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
                let mut printer = SimplePrinter::new(desktop, ((10, 10), (width - 10, height - 10)), &font);
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
        let prop_bottom = bottom - 32;

        draw_desktop(desktop);
        draw_dialog_box(desktop, (topleft, bottomright));

        let file_contents = fs::read_to_string("lorem-ipsum.txt");
        match file_contents {
            Err(e) => {
                let error_reason = format!("Could not open lorem-ipsum.txt because: {}", e);
                let mut printer = SimplePrinter::new(desktop, ((10, 10), (text_right, height - 10)), &font);
                printer.print(&error_reason);
            },
            Ok(contents) => {
                self.text_to_view.replace(Some(contents));
                self.print_file(desktop, &font);
            }
        }

        use bitblt::{BlitContext, BlitOp, blit_rect};
        draw_prop_gadget(desktop, ((prop_left, prop_top), (prop_right, prop_bottom)));
        desktop.framed_rectangle((prop_left, prop_bottom), (prop_right, prop_bottom+16), LINE_BLACK);
        desktop.framed_rectangle((prop_left, prop_bottom+16), (prop_right, prop_bottom+32), LINE_BLACK);
        {
            let mut bc = BlitContext::new(&arrow_bits_up, 2, &mut desktop.bits, (width >> 3) as usize);
            blit_rect(&mut bc, 0, 0, 8, 8, (prop_left + 4) as usize, (prop_bottom + 4) as usize, BlitOp::DandNotS);
        }
        {
            let mut bc = BlitContext::new(&arrow_bits_dn, 2, &mut desktop.bits, (width >> 3) as usize);
            blit_rect(&mut bc, 0, 0, 8, 8, (prop_left + 4) as usize, (prop_bottom + 20) as usize, BlitOp::DandNotS);
        }
    }
}

static arrow_bits_up: [u8; 16] = [
    0b00011000, 0b00000000,
    0b00011000, 0b00000000,
    0b00111100, 0b00000000,
    0b00111100, 0b00000000,
    0b01111110, 0b00000000,
    0b00011000, 0b00000000,
    0b00011000, 0b00000000,
    0b00011000, 0b00000000,
];

static arrow_bits_dn: [u8; 16] = [
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

