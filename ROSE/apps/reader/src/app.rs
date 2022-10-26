use std::fs;
use std::cell::RefCell;

use stencil::stencil::{Draw, Stencil};
use stencil::utils::{ draw_desktop, draw_dialog_box };
use stencil::simple_bitmap_font::SimpleBitmapFont;
use stencil::sysfont_bsw_9::SYSTEM_BITMAP_FONT;
use stencil::simple_printer::SimplePrinter;

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

        draw_desktop(desktop);
        draw_dialog_box(desktop, ((8, 8), (width - 8, height - 8)));

        let file_contents = fs::read_to_string("lorem-ipsum.txt");
        match file_contents {
            Err(e) => {
                let error_reason = format!("Could not open lorem-ipsum.txt because: {}", e);
                let mut printer = SimplePrinter::new(desktop, ((10, 10), (width - 10, height - 10)), &font);
                printer.print(&error_reason);
            },
            Ok(contents) => {
                self.text_to_view.replace(Some(contents));
                self.print_file(desktop, &font);
            }
        }
    }
}

