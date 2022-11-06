use crate::types::{Unit, Dimension};

/// A very simple bitmapped font descriptor.
pub struct SimpleBitmapFont<'a> {
    /// How many bytes wide the font bitmap is.
    pub span: usize,
    /// How tall the font is, in pixels.
    pub height: Unit,
    /// Where the baseline sits in the bitmap.
    /// `0` is the top-most pixel.
    pub baseline: Unit,
    /// Reference to the font bitmap raw data bytes.
    pub bits: &'a [u8],
    /// Reference to an array of horizontal coordinates.
    /// Each coordinate indicates the left edge
    /// of the glyph for the corresponding character index,
    /// adjusted for the lowest character.
    ///
    /// Note that index 0 corresponds to character `lowest_char`,
    /// and increases monotonically from there.
    ///
    /// The largest index supported is `highest_char`-`lowest_char`+2.
    /// At `highest_char`-`lowest_char`+1,
    /// you'll find the left edge of the *unknown* glyph
    /// (usually a non-descript rectangle).
    ///
    /// At `highest_char`-`lowest_char`+2,
    /// you'll find the pixel coordinate of the right-hand edge of the
    /// font bitmap.
    ///
    /// The *width of any glyph*
    /// can be determined by calculating the difference
    /// `left_edges[n+1] - left_edges[n]`,
    /// assuming `n` is scoped accordingly.
    pub left_edges: &'a [u16],
    /// Indicates the lowest character index supported in this font.
    /// Usually, this will be set to 32 (the ASCII space character).
    pub lowest_char: u8,
    /// Indicates the largest character index supported in this font.
    /// Usually, this will be set to 127 for fonts that cover the
    /// 96 graphic characters in the ASCII standard.
    pub highest_char: u8,
}

/// Compute the pixel width of the string,
/// if a printer were to draw the string onto a stencil.
///
/// The `text` argument should not include control characters
/// like new-lines or carriage-returns.
/// All control characters will be treated as glyphs to be printed.
/// If the font doesn't support these glyphs,
/// the unknown glyph will be used.
pub fn text_width(text: &str, font: &SimpleBitmapFont) -> Dimension {
    text.bytes().map(|b| {
        // If not representable in the glyph set of the font, assume the undefined character glyph,
        // which by definition, is always at highest_char+1 mod 256.
        let highest_character = font.highest_char;
        let lowest_character = font.lowest_char;
        let mut glyph_index = b as usize;

        if (b < lowest_character) || (b > highest_character) {
            glyph_index = (highest_character as usize).overflowing_add(1).0;
        }
        glyph_index -= lowest_character as usize;

        // Let's expand this to a valid array index.

        let left_edge = font.left_edges[glyph_index];
        let right_edge = font.left_edges[glyph_index + 1];
        let glyph_width = right_edge - left_edge;

        glyph_width as Dimension
    }).sum()
}

