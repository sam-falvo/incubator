//! Bit-blit support for monochrome bitmaps.
//!
//! # Bitmaps
//!
//! A bitmap is described as a slice of bytes (`[u8]`), a width, a height, and sometimes a "modulo".
//! The slice of bytes covers a region of memory which encompasses the entire rectangular region of
//! a bitmap, as shown below.
//!
//! ```text
//! let src: &[u8];
//!      |
//! .----'
//! |
//! V
//! +----+====+====+====+====+----+----+----+ .
//! |  0 #  1 |  2 |  3 |  4 #  5 |  6 |  7 |  |
//! +----+----+----+----+----+----+----+----+  |
//! |  8 #  9 | 10 | 11 | 12 # 13 | 14 | 15 |  |
//! +----+----+----+----+----+----+----+----+   > bitmap height
//! | 16 # 17 | 18 | 19 | 20 # 21 | 22 | 23 |  |
//! +----+----+----+----+----+----+----+----+  |
//! | 24 # 25 | 26 | 27 | 28 # 29 | 30 | 31 |  |
//! +----+====+====+====+====+----+----+----+ '
//!      `---------v---------'
//!           bitmap width
//! ```
//!
//! **NOTE:**
//! It is easiest if the slice refers directly to the upper-lefthand corner of a bitmap.
//! This allows you to refer
//! to subrectangles of the bitmap
//! without the need for coordinate translation.
//! However,
//! as long as you're careful with coordinate translation,
//! the slice doesn't strictly have to refer to the upper left-hand corner of a bitmap.
//! The only hard requirement is that the slice encompasses every byte of the rectangle.
//! 
//! The *width* indicates how many *bytes* (note: not pixels!) makes up a single raster line of a
//! bitmap.  In the example above, the width would be 4, since each raster line takes up 4 bytes.
//!
//! The *height* indicates how many raster lines of the bitmap we wish to consider in the blit
//! operation.  In the example above, the height is also 4.
//!
//! If you use the lower-level facilities of this library, you'll also need to be aware of
//! something called a *modulo*.  This is a measure of the number of bytes from the *end* of one
//! raster line to the *start* of another.  This is what allows the bit-blit library to consider
//! sub-rectangles of a larger bitmap.
//!
//! ```text
//! +----+====+====+====+====+----+----+----+
//! |  0 #  1 |  2 |  3 |  4 #  5 |  6 |  7 |  5 + 4 = 9; therefore, the modulo = 4.
//! +----+----+----+----+----+----+----+----+
//!                            |
//! --------.                  `------------
//!         |
//!         V
//! +----+----+----+----+----+----+----+----+
//! |  8 #  9 | 10 | 11 | 12 # 13 | 14 | 15 |  13 + 4 = 17
//! +----+----+----+----+----+----+----+----+
//!                            |
//! --------.                  `------------
//!         |
//!         V
//! +----+----+----+----+----+----+----+----+
//! | 16 # 17 | 18 | 19 | 20 # 21 | 22 | 23 |  21 + 4 = 25
//! +----+----+----+----+----+----+----+----+
//!                            |
//! --------.                  `------------
//!         |
//!         V
//! +----+----+----+----+----+----+----+----+
//! | 24 # 25 | 26 | 27 | 28 # 29 | 30 | 31 |  29 + 4 = 33; however, this falls outside
//! +----+====+====+====+====+----+----+----+  the sub-rectangle, so we no longer care.
//! ```
//!
//! Note that it is entirely fine for a modulo to be zero;
//! in this case, each raster line is stored back to back in memory with no intervening gaps.
//!
//! Sometimes, you'll see mentioned a *span*.
//! Spans are just the sum of a width and a modulo,
//! as illustrated below.
//!
//! ```text
//! +----+====+====+====+====+----+----+----+
//! |  0 #  1 |  2 |  3 |  4 #  5 |  6 |  7 |  1 + 4 (width) + 4 (modulo) = 9; therefore, the span = 4.
//! +----+----+----+----+----+----+----+----+
//!         |                   :
//!         `------------------- ------------
//! --------.
//!         |
//!         V
//! +----+----+----+----+----+----+----+----+
//! |  8 #  9 | 10 | 11 | 12 # 13 | 14 | 15 |
//! +----+----+----+----+----+----+----+----+
//!  . . . 
//! ```
//!
//! # What is a Blit?
//!
//! The noun, *blit*, refers to the results of combining two or more sources of bitmap data to
//! yield a resulting destination bitmap.  The verb, *blit*, refers to the *act* of computing the
//! above result.
//!
//! The blitter algorithm implemented in this library considers two bitmap sources: S and D.
//! The S source is purely an input bitmap.
//! The D source is not only an input, but also is the destination for all blit operations.
//! 
//! Bitmap data from S can be shifted up to seven bits either to the left or to the right.
//! After shifting, masking is applied to remove unwanted bits.
//! This mechanism allows the bit-blitter implementation to not only work with sub-rectangles on
//! byte boundaries, but also on *bit* boundaries as well.
//!
//! The D source is used as-is, without any masking or shifting.
//!
//! # BlitContext
//!
//! In order to use the bit blitter in this library, you'll need to first create a [[BlitContext]]
//! structure.  This structure contains the working state of the blitter implementation.
//! A programmer won't typically need to know the implementation details; however,
//! if the high-level [[blit_rect]] function is not convenient to use,
//! manipulating the BlitContext structure will be required before invoking the lower-level
//! functions.
//!
//! **NOTE:** If you need to use the lower-level functions, you are at greater risk for API
//! breakage should the implementation details change.  When and where possible, you'll *really*
//! want to use [[blit_rect]] for your blitting needs.
//!
//! # Blitting More than Bits
//!
//! This library is intended for use with monochrome bitmaps.
//! For color bitmaps, the most convenient representation is through the use of *bit-planes*.
//! Imagine stacked bitmaps, where each bitmap contributes its own bits to determine a single
//! pixel's color.
//! However, it'll be up to the caller to interpret the color bitmaps appropriately.
//!
//! ```text
//!                .----------------------- color 0
//!               /   .-------------------- color 1
//!              /   /   .----------------- color 2
//!             /   /   /   .-------------- color 3
//!            /   /   /   /   .----------- color 4
//!           /   /   /   /   /   .-------- color 5
//!          /   /   /   /   /   /   .----- color 6
//!         /   /   /   /   /   /   /   .-- color 7
//!        /   /   /   /   /   /   /   /
//!     +---+---+---+---+---+---+---+---+---
//!     | 0 | 0 | 0 | 0 | 1 | 1 | 1 | 1 | ...
//!   +---+---+---+---+---+---+---+---+---
//!   | 0 | 0 | 1 | 1 | 0 | 0 | 1 | 1 | ...
//! +---+---+---+---+---+---+---+---+---
//! | 0 | 1 | 0 | 1 | 0 | 1 | 0 | 1 | ...
//! ```
//!
//! However, with clever configuration of the [[BlitContext]] structure,
//! it can also be made to work with 2bpp or 4bpp chunky pixel maps.
//! In particular, shifts and masks would need to be aligned to 2- or 4-bit boundaries;
//! however, it should still work well.
//!
//! ```text
//! +---+---+---+---+---+---+---+---+---
//! | 0 : 0 | 0 : 1 | 1 : 0 | 1 : 1 | ...
//! +---+---+---+---+---+---+---+---+---
//!  `-----' `-----' `-----' `-----'
//!     |       |       |       |
//!     |       |       |       `---- color 3
//!     |       |       `------------ color 2
//!     |       `-------------------- color 1
//!     `---------------------------- color 0
//! ```
//!
//! While this blitter library can also be used to blit 8bpp or larger display depths,
//! one must ask why they'd want to.  At these color depths, it's faster and much simpler
//! to just shuffle bytes between byte slices directly.

/// Indicates how to combine the source data and destination data
/// when performing a bitblt operation.  If you consider the four minterms
/// one can combine two sources of data with (!D & !S, !D & S, D & !S, and D & S),
/// there are sixteen operations to select from.
///
/// Black will always fill destination bits with zeros, as it selects no minterms at all.
/// White will always fill destination bits with ones, as it selects all of them.
#[repr(usize)]
#[derive(Debug)]
pub enum BlitOp {
    Black,
    Nor,
    NotDandS,
    NotD,
    DandNotS,
    NotS,
    Xor,
    Nand,
    And,
    Xnor,
    S,
    SorNotD,
    D,
    DorNotS,
    Or,
    White,
}

/// Encapsulates the current state of a bitblt operation.  This structure should be initialized
/// according to the type of blit operation you wish to perform prior to invoking any other bitblt
/// functions.
pub struct BlitContext<'a> {
    //--- Byte-wise Fields

    /// Reference to a slice of bytes containing source data.
    pub s_bits: &'a [u8],

    /// Index of the next byte to be read from `s_bits`.
    pub s_ptr: usize,

    /// Number of bytes to add to `s_ptr` to advance to the byte directly below it in the source
    /// bitmap.
    pub s_span: usize,

    /// The number of bits to shift data read from `s_bits`.
    pub s_shift: u8,

    /// After reading the next byte from `s_bits` and
    /// shifting by `s_shift` bits, this mask is logically-ANDed
    /// to yield the final source value.
    pub s_mask: u8,

    /// The previous byte read from `s_bits`.  
    pub s_data: u8,

    /// Mutable reference to a slice of bytes containing the
    /// destination data.
    pub d_bits: &'a mut [u8],

    /// Index of the next byte to be written to `d_bits`.
    pub d_ptr: usize,

    /// Number of bytes to add to `d_ptr` to advance to the byte directly below it in the
    /// destination bitmap.
    pub d_span: usize,

    /// The logical operation to apply to the source and destination data.
    pub operation: BlitOp,

    //--- Line-wise Fields

    /// Mask to apply when blitting the first column of a raster line.
    /// For ascending blits, this will be on the left-hand edge;
    /// for descending blits, on the right-hand edge.
    pub s_fc_mask: u8,

    /// Mask to apply when blitting the last column of a raster line.
    /// For ascending blits, this will be on the right-hand edge;
    /// for descending blits, on the left-hand edge.
    pub s_lc_mask: u8,
}

impl<'a> BlitContext<'a> {
    /// Create a new BlitContext with reasonable defaults.
    ///
    /// Currently, these defaults assume no shift,
    /// first and last column masks are set to 0xFF,
    /// the logical operation is set to [[BlitOp]]::Or,
    /// and all pointers refer to index 0 of their respective bits slices.
    ///
    /// These defaults will not be sufficient for most blit operations, however.
    /// For this reason, you should use functions like [[blit_rect]] to configure
    /// the context and perform the blit operation in a single step.
    pub fn new(src: &'a [u8], src_span: usize, dst: &'a mut [u8], dst_span: usize) -> Self {
        BlitContext {
            s_bits: src,
            s_ptr: 0,
            s_span: src_span,
            s_shift: 0,
            s_data: 0,
            s_mask: 0xFF,
            s_fc_mask: 0xFF,
            s_lc_mask: 0xFF,

            d_bits: dst,
            d_ptr: 0,
            d_span: dst_span,

            operation: BlitOp::Or,
        }
    }
}

/// Given a source and a destination bitmap,
/// compute a new destination byte from
/// a single source byte and a destination byte.
///
/// This function encapsulates the core of the bit-blit algorithm.
/// It works as follows:
///
/// Read a byte from the source channel (`s_bits`);
/// shift it to the right by `s_shift` bits and
/// merge with the discarded bits of the previously read byte.
/// Then, apply the mask in `s_mask`.
///
/// Next, read a byte from the destination channel (`d_bits`),
/// and combine with the shifted and masked byte from above
/// according to the selected operation (see [[BlitOp]]).
/// Write the combined value back to the destination.
///
/// Increment the source and destination pointers.
#[inline(always)]
pub fn blit_byte_ascending(bc: &mut BlitContext) {
    let raw_s = bc.s_bits[bc.s_ptr] & bc.s_mask;
    let s = ((raw_s as u16 >> bc.s_shift) | ((bc.s_data as u16) << (8 - bc.s_shift))) as u8;
    let d = bc.d_bits[bc.d_ptr];

    let d = match bc.operation {
        BlitOp::Black => 0u8,
        BlitOp::Nor => !(s | d),
        BlitOp::NotDandS => !d & s,
        BlitOp::NotD => !d,
        BlitOp::DandNotS => d & !s,
        BlitOp::NotS => !s,
        BlitOp::Xor => s ^ d,
        BlitOp::Nand => !(s & d),
        BlitOp::And => s & d,
        BlitOp::Xnor => !(s | d),
        BlitOp::S => s,
        BlitOp::SorNotD => s | !d,
        BlitOp::D => d,
        BlitOp::DorNotS => d | !s,
        BlitOp::Or => s | d,
        BlitOp::White => 0xFFu8,
    };

    bc.d_bits[bc.d_ptr] = d;
    bc.d_ptr = bc.d_ptr.overflowing_add(1).0;
    bc.s_ptr = bc.s_ptr.overflowing_add(1).0;
    bc.s_data = raw_s;
}

/// Given a source and a destination bitmap,
/// compute a new destination byte from
/// a single source byte and a destination byte.
///
/// This function encapsulates the core of the bit-blit algorithm.
/// It works as follows:
///
/// Read a byte from the source channel (`s_bits`);
/// shift it to the left by `s_shift` bits and
/// merge with the discarded bits of the previously read byte.
/// Then, apply the mask in `s_mask`.
///
/// Next, read a byte from the destination channel (`d_bits`),
/// and combine with the shifted and masked byte from above
/// according to the selected operation (see [[BlitOp]]).
/// Write the combined value back to the destination.
///
/// Decrement the source and destination pointers.
#[inline(always)]
pub fn blit_byte_descending(bc: &mut BlitContext) {
    let raw_s = bc.s_bits[bc.s_ptr] & bc.s_mask;
    let s = (((raw_s as u16) << bc.s_shift) | ((bc.s_data as u16) >> (8 - bc.s_shift))) as u8;
    let d = bc.d_bits[bc.d_ptr];

    let d = match bc.operation {
        BlitOp::Black => 0u8,
        BlitOp::Nor => !(s | d),
        BlitOp::NotDandS => !d & s,
        BlitOp::NotD => !d,
        BlitOp::DandNotS => d & !s,
        BlitOp::NotS => !s,
        BlitOp::Xor => s ^ d,
        BlitOp::Nand => !(s & d),
        BlitOp::And => s & d,
        BlitOp::Xnor => !(s | d),
        BlitOp::S => s,
        BlitOp::SorNotD => s | !d,
        BlitOp::D => d,
        BlitOp::DorNotS => d | !s,
        BlitOp::Or => s | d,
        BlitOp::White => 0xFFu8,
    };

    bc.d_bits[bc.d_ptr] = d;
    bc.d_ptr = bc.d_ptr.overflowing_sub(1).0;
    bc.s_ptr = bc.s_ptr.overflowing_sub(1).0;
    bc.s_data = raw_s;
}

/// Performs a blit operation on a single raster line
/// (or vector of bytes)
/// defined as `width` bytes wide.
///
/// The source raster line is assumed to start at `bc.s_ptr` in `bc.s_bits`.
/// The destination raster line is assumed to start at `bc.d_ptr` in `bc.d_bits`.
/// The blit operation is performed in an ascending order; that is,
/// pointers increment and source data is shifted to the right (if shifted at all).
pub fn blit_line_ascending(bc: &mut BlitContext, width: usize) {
    match width {
        0 => (),
        1 => {
            bc.s_mask = bc.s_fc_mask & bc.s_lc_mask;
            blit_byte_ascending(bc);
        }
        _ => {
            bc.s_mask = bc.s_fc_mask;
            blit_byte_ascending(bc);

            bc.s_mask = 0xFF;
            for _ in 1 .. width - 1 {
                blit_byte_ascending(bc);
            }

            bc.s_mask = bc.s_lc_mask;
            blit_byte_ascending(bc);
        }
    }
}

/// Performs a blit operation on a single raster line
/// (or vector of bytes)
/// defined as `width` bytes wide.
///
/// The source raster line is assumed to start at `bc.s_ptr` in `bc.s_bits`.
/// The destination raster line is assumed to start at `bc.d_ptr` in `bc.d_bits`.
/// The blit operation is performed in a descending order; that is,
/// pointers decrement and source data is shifted to the left (if shifted at all).
pub fn blit_line_descending(bc: &mut BlitContext, width: usize) {
    match width {
        0 => (),
        1 => {
            bc.s_mask = bc.s_fc_mask & bc.s_lc_mask;
            blit_byte_descending(bc);
        }
        _ => {
            bc.s_mask = bc.s_fc_mask;
            blit_byte_descending(bc);

            bc.s_mask = 0xFF;
            for _ in 1 .. width - 1 {
                blit_byte_descending(bc);
            }

            bc.s_mask = bc.s_lc_mask;
            blit_byte_descending(bc);
        }
    }
}

/// Combines a sub-rectangle of the source bitmap with a corresponding sub-rectangle of the
/// destination bitmap, and writes the result into the destination bitmap.
///
/// The `src_left` and `src_top` arguments
/// provides the pixel coordinates of the top, left-hand corner
/// of the sub-rectangle in the source bitmap.
/// Likewise, `dst_left` and `dst_top`
/// do the same for the sub-rectangle in the destination bitmap.
/// Note that the width and height of the sub-rectangle
/// are determined by `src_right` and `src_bottom`,
/// which provides the pixel coordinates for the lower right-hand corner
/// of the source sub-rectangle.
///
/// Remember that `src_right` and `src_bottom` are *exclusive* coordinates, not inclusive;
/// they indicate coordinates that lie between pixels, and are not pixel labels themselves.
/// Thus, a one-pixel sub-rectangle is described using coordinates (L, T)-(L+1, T+1).
///
/// The [[BlitContext]]'s `s_data` field should be initialized with a value that supplies data bits
/// when shifting the first source byte in each line.  The [[new]] function sets this field to 0,
/// which is almost always correct.  Note: Whether this data supplies bits for the left-hand or
/// right-hand edges depends whether the blit operation occurs in ascending (left-hand edge) or
/// descending (right-hand edge) mode.
///
/// # Panics
///
/// It is critical that the dimensions of the source sub-rectangle are made such that the
/// sub-rectangle will also fit in the destination bitmap.  If not, slice indexing panics may
/// occur.
pub fn blit_rect(
    bc: &mut BlitContext,
    src_left: usize,
    src_top: usize,
    src_right: usize,
    src_bottom: usize,
    dst_left: usize,
    dst_top: usize,
    op: BlitOp,
) {
    let (width_in_bytes, height, ascending_mode) = prepare_blit_rect(bc, src_left, src_top, src_right, src_bottom, dst_left, dst_top, op);
    perform_blit_rect(bc, width_in_bytes, height, ascending_mode);
}

/// Configures a [[BlitRect]] structure for a rectangular blit operation.
///
/// You generally won't be needing this function; it's intended purpose is to facilitate unit
/// testing.
///
/// It answers with a tuple of three results:
/// - the width (in bytes) and height (in pixels) of the blit, and,
/// - a flag indicating whether the blit operation is ascending (true) or descending (false).
///
/// The [[BlitContext]] will have its fields initialized appropriately for performing a rectangular
/// blit operation.  The results of this function can be fed directly into [[perform_blit_rect]] to
/// actually perform the blit operation.
#[inline(always)]
pub fn prepare_blit_rect(
    bc: &mut BlitContext,
    src_left: usize,
    src_top: usize,
    src_right: usize,
    src_bottom: usize,
    dst_left: usize,
    dst_top: usize,
    op: BlitOp,
) -> (usize, usize, bool) {
    // The blitter operates on bytes; we therefore need to know the *byte* coordintes of our source
    // and destination sub-rectangles.
    let height = src_bottom - src_top;
    let width_in_pixels = src_right - src_left;

    let inclusive_src_right = src_right - 1;
    let s_left_column = src_left >> 3;
    let s_right_column = inclusive_src_right >> 3;
    let s_right_pixel_in_byte = src_right & 7;

    let dst_right = dst_left + width_in_pixels;
    let dst_bottom = dst_top + height;
    let inclusive_dst_right = dst_right - 1;
    let d_left_column = dst_left >> 3;
    let d_right_column = inclusive_dst_right >> 3;

    // If we place the source and destination bitmaps so that their upper lefthand corner bytes
    // overlap, then the relative placement of source and destination sub-rectangles in these bytes
    // are used to determine if we should use ascending- or descending-mode blits.  If we're
    // shifting bitmap data to the right, we need to perform the blit operation in ascending mode.
    // Otherwise, descending.
    let s_left_pixel_in_byte = src_left & 7;
    let d_left_pixel_in_byte = dst_left & 7;
    let ascending_mode = s_left_pixel_in_byte < d_left_pixel_in_byte;

    // The blit always takes an integral number of bytes to process.  Sometimes, due to the need
    // for shifting, the number of destination bytes to process is greater than the number of
    // source bytes, or vice versa.  Thus, we always take the maximum width of either the source or
    // destination subrectangle.
    // 
    // Observation: the source and destination widths should never differ by more than one.
    let s_width_in_bytes = s_right_column - s_left_column + 1;
    let d_width_in_bytes = d_right_column - d_left_column + 1;
    let width_in_bytes = s_width_in_bytes.max(d_width_in_bytes);

    // Determine the upper lefthand byte offset of the source and destination subrectangles.
    // If descending, we must point at the lower righthand corner instead.
    let s_ptr: usize;
    let d_ptr: usize;
    let s_fc_mask: u8;
    let s_lc_mask: u8;

    if ascending_mode {
        s_ptr = (bc.s_span * src_top) + s_left_column;
        d_ptr = (bc.d_span * dst_top) + d_left_column;
        s_fc_mask = ASCENDING_FIRST_MASKS[s_left_pixel_in_byte as usize];
        s_lc_mask = ASCENDING_LAST_MASKS[s_right_pixel_in_byte as usize];
    } else {
        s_ptr = (bc.s_span * (src_bottom - 1)) + s_left_column + width_in_bytes - 1;
        d_ptr = (bc.d_span * (dst_bottom - 1)) + d_left_column + width_in_bytes - 1;
        s_fc_mask = DESCENDING_FIRST_MASKS[s_right_pixel_in_byte as usize];
        s_lc_mask = DESCENDING_LAST_MASKS[s_left_pixel_in_byte as usize];
    }

    // Configure the BlitContext, then hand control over to the ascending or
    // descending blit line functions for actual processing.
    bc.s_ptr = s_ptr;
    bc.s_shift = (d_left_pixel_in_byte as isize - s_left_pixel_in_byte as isize).abs() as u8;
    bc.d_ptr = d_ptr;
    bc.operation = op;
    bc.s_fc_mask = s_fc_mask;
    bc.s_lc_mask = s_lc_mask;

    (width_in_bytes, height, ascending_mode)
}

static ASCENDING_FIRST_MASKS: [u8; 8] = [
    0b11111111, 0b01111111, 0b00111111, 0b00011111, 0b00001111, 0b00000111, 0b00000011,
    0b00000001,
];

static ASCENDING_LAST_MASKS: [u8; 8] = [
    0b00000000, 0b10000000, 0b11000000, 0b11100000, 0b11110000, 0b11111000, 0b11111100,
    0b11111110,
];

// Sacrifice some memory for easier to maintain source code.
// A "sufficiently smart compiler" would know to create aliases here anyway.
static DESCENDING_FIRST_MASKS: [u8; 8] = ASCENDING_LAST_MASKS;
static DESCENDING_LAST_MASKS: [u8; 8] = ASCENDING_FIRST_MASKS;

/// Drives a rectangular blit operation to completion.
#[inline(always)]
pub fn perform_blit_rect(bc: &mut BlitContext, width_in_bytes: usize, height: usize, ascending_mode: bool) {
    let data_preload = bc.s_data;
    if ascending_mode {
        for _ in 0..height {
            bc.s_data = data_preload;
            blit_line_ascending(bc, width_in_bytes);
        }
    } else {
        for _ in 0..height {
            bc.s_data = data_preload;
            blit_line_descending(bc, width_in_bytes);
        }
    }
}

#[cfg(test)]
mod tests;
