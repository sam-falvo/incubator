//! Supports immediate-mode drawing into off-screen bitmaps, called stencils.
//!
//! Stencils are off-screen, monochrome bitmaps.
//! They are primarily intended
//! to be used as masks
//! through which color is applied
//! to an underlying surface.
//! However,
//! for hardware which is incapable of supporting color,
//! stencils can also be rendered as-is.
//!
//! Unless otherwise documented,
//! the coordinate space for manipulating stencils includes the left or top coordinate, but
//! excludes the right or bottom coordinate.
//! You can think of coordinates as falling *between* pixels,
//! rather than naming them directly.
//! For example, here's the coordinate space for a 6x3 stencil:
//!
//! ```text
//!   0  1  2  3  4  5  6
//! 0 +--+--+--+--+--+--+
//!   |##|##|##|##|##|##|
//! 1 +--+--+--+--+--+--+
//!   |##|##|##|##|##|##|
//! 2 +--+--+--+--+--+--+
//!   |##|##|##|##|##|##|
//! 3 +--+--+--+--+--+--+
//! ```
//!
//! Thus,
//! if you wanted to draw a rectangle
//! in the upper-lefthand corner
//! that was 3 pixels wide and 2 pixels tall,
//! you would invoke `filled_rectangle((0,0), (3,2))`.
//! To do the same with in the lower-righthandl corner,
//! you'd invoke `filled_rectangle((3,1), (6,3))`.
//!
//! Where you do need to identify a particular pixel or row of pixels,
//! the *inclusive* coordinate is used for this purpose.
//! For example,
//! the first, second, and third row of pixels in the previous example
//! are identified as row 0, row 1, and row 2, respectively.
//! Note that row 3 does not exist here.
//!
//! Similar rules apply for columns as well.
//! In the above example, columns 0 through 5 exist,
//! but, column 6 does not.

/// A single coordinate component.
pub type Unit = isize;

/// A coordinate.
pub type Point = (Unit, Unit);

/// A dimension component.
pub type Dimension = isize;

/// A Stencil encapsulates a bitmapped image.
pub struct Stencil {
    /// (Width, Height) of the stencil, in dots.
    dimensions: (Dimension, Dimension),

    /// The storage for the raw bits of the stencil.
    bits: Vec<u8>,
}

static LEFT_MASKS: [u8; 8] = [ 0xFF, 0x7F, 0x3F, 0x1F, 0x0F, 0x07, 0x03, 0x01, ];
static RIGHT_MASKS: [u8; 8] = [ 0x80, 0xC0, 0xE0, 0xF0, 0xF8, 0xFC, 0xFE, 0xFF, ];

impl Stencil {
    /// Create a new stencil with the dimensions (width, height) provided.
    /// If the dimensions are inappropriate (e.g., a width which would overflow a signed integer),
    /// answer with None.  Otherwise, yield a Stencil instance.
    pub fn new_with_dimensions(width: usize, height: usize) -> Option<Self> {
        // Confirm that width and height are safe to use with signed-integer fields.
        // If not, return None.

        let largest_safe_value: usize = Dimension::MAX as usize;

        if width > largest_safe_value {
            return None;
        }

        if height > largest_safe_value {
            return None;
        }

        // Otherwise, attempt to allocate memory for the bitmap, and record
        // the width and height.  Return the stencil.

        let span = (width + 7) >> 3;
        let size = span * height;

        let mut bits = Vec::with_capacity(size);
        bits.resize(size, 0);

        Some(Self {
            dimensions: (width as Dimension, height as Dimension),
            bits,
        })
    }

    /// Answer with the number of bytes a single row of pixels takes in memory.
    pub fn get_span(&self) -> usize {
        let width = self.dimensions.0 as usize;

        (width + 7) >> 3
    }

    /// Borrow the buffer containing the bitmapped image as a slice of bytes.
    /// 
    /// Within each byte,
    /// bit 7 presents the left-most pixel value while bit 0 presents the right-most.
    ///
    /// Each row of the bitmap is laid out sequentially,
    /// starting with the left-most byte and ending with the right-most byte.
    /// Unused pixels in the right-most byte are ignored.
    ///
    /// ```text
    ///     Pixel number --->
    ///                                               1   1   1   1   1   1   ...
    ///       0   1   2   3   4   5   6   7   8   9   0   1   2   3   4   5 
    ///     +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+----
    /// Bit | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 | ...
    ///     +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+----
    ///     \______________  ______________/ \_____________  ______________/ \___
    ///                    \/                              \/
    ///                  Byte N                         Byte N+1
    /// ```
    ///
    /// This is referred to as *big-endian* bitmap layout,
    /// because big-endian processors
    /// can freely shift bits left or right
    /// using 16-bit or wider instructions.
    /// Little-endian processors
    /// will need to use byte-swap operations to do the same,
    /// often finding it faster to just stick with byte operations.
    ///
    /// Each row is laid out sequentially, from top to bottom.
    /// Unlike some graphics display hardware,
    /// such as that found in the Commodore 8-bit family of computers,
    /// no pseudo-tiling occurs.
    pub fn borrow_bits(&self) -> &[u8] {
        &self.bits
    }

    /// Draw a filled rectangle with the given pattern.
    pub fn filled_rectangle(&mut self, upper_left: Point, lower_right: Point, pattern: &[u8; 8]) {
        let (left, top) = upper_left;
        let (right, bottom) = lower_right;
        let (width, height) = (right - left, bottom - top);

        if (width <= 0) || (height <= 0) { return }

        for y in 0..height {
            self.horizontal_line((left, top + y), right, pattern[(y & 7) as usize]);
        }
    }

    /// Draw a horizontal line with the given pattern.
    pub fn horizontal_line(&mut self, left: Point, mut right: Unit, pattern: u8) {
        let (mut left, top) = left;
        let (width, height) = self.dimensions;

        // Perform basic clipping to the stencil.
        //
        // First, make sure the horizontal line is neither above nor below the stencil.
        if (top < 0) || (top >= height) { return }

        // Next, make sure the left coordinate is to the left of the right coordinate.
        // Swap them if necessary.
        if left > right {
            let t = right;
            right = left;
            left = t;
        }

        // Knowing left <= right, check to see if the line is too far to the left or right to be
        // visible on the stencil.
        if (right < 0) || (left >= width) { return }

        // Make sure that the line has a width of at least one pixel.
        if left == right { return }

        // Constrain the coordinates to the stencil.
        left = left.max(0);
        right = right.min(width);

        // We know right > left and right-left >= 1.
        // Decrement right to use inclusive coordinates instead of exclusive.
        let right = right - 1;
        let span = (width + 7) >> 3;

        let left_byte = ((span * top) + (left >> 3)) as usize;
        let right_byte = ((span * top) + (right >> 3)) as usize;
        let left_mask = LEFT_MASKS[(left & 7) as usize];
        let right_mask = RIGHT_MASKS[(right & 7) as usize];

        let mut x = left_byte;
        while x <= right_byte {
            let mut combined_mask = 0xFF;
            if x == left_byte {
                combined_mask &= left_mask;
            }
            if x == right_byte {
                combined_mask &= right_mask;
            }

            let original_byte = self.bits[x];
            let desired_bits = combined_mask & pattern;
            let unaffected_bits = !combined_mask & original_byte;
            let new_byte = unaffected_bits | desired_bits;
            self.bits[x] = new_byte;

            x = x + 1;
        }
    }
}

