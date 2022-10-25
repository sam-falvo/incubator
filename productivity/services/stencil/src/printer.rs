use bitblt::{BlitOp, BlitContext, blit_rect};
use crate::types::{Rect, Unit, Point};
use crate::simple_bitmap_font::SimpleBitmapFont;
use crate::stencil::Stencil;

/// This structure is an attempt to emulate a very basic dot-matrix printer,
/// which prints onto a Stencil instead of actual paper.  This is a useful
/// abstrction for controlling how strings are printed to a stencil.
pub struct SimplePrinter<'a, 'b> {
    /// The stencil this printer will be printing on to.
    pub stencil: &'a mut Stencil,

    /// Determines the top, bottom, left, and right-hand margins relative
    /// to the stencil on which this printer is printing to.
    pub margins: Rect,

    /// The current font that text will be printed in.
    pub font: &'b SimpleBitmapFont<'b>,

    /// The location of the current print "head".  Note that this locates the
    /// *baseline*, not the top-left corner of the next glyph.
    pub head: Point,
}

impl<'a, 'b> SimplePrinter<'a, 'b> {
    /// Creates a new SimplePrinter.
    ///
    /// The `stencil` is the surface on which the printer will render text.
    ///
    /// The `margins` rectangle delineates the "page" within which the printer will constrain its
    /// virtual print head to.
    ///
    /// The `font` indicates the font the printer will use for printing.
    ///
    /// After creation, the print head will be placed in the upper left-hand corner of the margins
    /// rectangle.
    pub fn new(stencil: &'a mut Stencil, margins: Rect, font: &'b SimpleBitmapFont<'b>) -> Self {
        let (left, top) = (margins.0.0, margins.0.1 + font.baseline);

        Self {
            stencil,
            margins,
            font,

            head: (left, top),
        }
    }

    /// Emulate a carriage return.
    fn carriage_return(&mut self) {
        self.head.0 = self.margins.0.0;
    }

    /// Emulate a line feed operation.
    fn line_feed(&mut self) {
        self.head.1 += self.font.height;
    }

    /// Wrap a character `b` from the right-hand side of the margin rectangle to the left-hand side
    /// on the next line.
    fn line_wrap(&mut self, b: u8) {
        self.carriage_return();
        self.line_feed();
        self.print_byte(b)
    }

    /// Print a single byte, including control codes.
    fn handle_byte(&mut self, b: u8) {
        match b {
            0x0D => self.carriage_return(),
            0x0A => { self.carriage_return(); self.line_feed(); },
            _ => self.print_byte(b),
        }
    }
}

/// All printer-compatible types can implement this trait.
pub trait Printer {
    /// Print a `text` string.  The layout of the text will depend on the specific printer you're
    /// using.
    fn print(&mut self, text: &str);

    /// Print a single byte `b`, handling only simple control codes like carriage return, etc.
    fn print_byte(&mut self, b: u8);
}

impl Printer for SimplePrinter<'_, '_> {
    fn print(&mut self, text: &str) {
        let mut text_iter = text.bytes();
        loop {
            let some_b = text_iter.next();
            match some_b {
                None => break,
                Some(b) => self.handle_byte(b),
            }
        }
    }

    fn print_byte(&mut self, b: u8) {
        // If not representable in the glyph set of the font, assume the undefined character glyph,
        // which by definition, is always at highest_char+1 mod 256.
        let highest_character = self.font.highest_char;
        let lowest_character = self.font.lowest_char;
        let mut glyph_index = b as usize;

        if (b < lowest_character) || (b > highest_character) {
            glyph_index = (highest_character as usize).overflowing_add(1).0;
        }
        glyph_index -= lowest_character as usize;

        // Let's expand this to a valid array index.

        let left_edge = self.font.left_edges[glyph_index];
        let right_edge = self.font.left_edges[glyph_index + 1];
        let glyph_width = right_edge - left_edge;

        // Check to see if we can print this character on the stencil
        // completely.  If not, defer to a callback, which may or may not
        // re-enter print_byte.
        let (x, y) = self.head;
        let new_cursor_position = x + glyph_width as Unit;
        let right_hand_margin = self.margins.1.0;
        if new_cursor_position >= right_hand_margin {
            self.line_wrap(b);
            return;
        }

        let glyph_bottom = self.head.1 - self.font.baseline + self.font.height;
        if glyph_bottom >= self.margins.1.1 {
            // Bottom of glyph falls below bottom of the margin bottom.
            return;
        }

        // Blit the character glyph onto the stencil.
        //
        // Because of a bug in bitblt that I do not know how to resolve yet, we must do this in two
        // steps.  First, we must blit the glyph into a temporary holding stencil that is sized large
        // enough for the glyph, but which holds the *pre-shifted* version of the glyph.

        let x_in_byte = (x & 7) as usize;

        // TODO: replace this with a higher-level interface.
        let mut tmp_buf: [u8; 48] = [0; 48]; // Big enough for fonts up to 16 pixels tall, and with characters no wider than 16px.
        let mut bc = BlitContext::new(self.font.bits, self.font.span, &mut tmp_buf, 3);
        blit_rect(
            &mut bc,
            left_edge as usize,
            0,
            right_edge as usize,
            self.font.height as usize,
            x_in_byte,
            0,
            BlitOp::S,
        );

        // Next, blit the pre-shifted glyph onto the destination bitmap with the operation requested by
        // the caller.  This requires resetting the BlitContext to refer to the correct stencil
        // bitmaps.
        bc.s_bits = &(*bc.d_bits);
        bc.s_span = 3;

        let stencil_width = self.stencil.dimensions.0;
        bc.d_bits = &mut self.stencil.bits;
        bc.d_span = ((stencil_width + 7) >> 3) as usize;

        let top = (y - self.font.baseline) as usize;
        blit_rect(
            &mut bc,
            x_in_byte,
            0,
            x_in_byte + glyph_width as usize,
            self.font.height as usize,
            x as usize,
            top,
            BlitOp::DandNotS,
        );

        self.head.0 = new_cursor_position;
    }
}

