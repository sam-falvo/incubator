// Working at the level of individual bytes,
// these operations are the lowest level of abstraction for blitting something.
mod byte_level {
    use crate::{blit_byte_ascending, blit_byte_descending, BlitContext};

    // Given that you have a properly initialized BlitContext structure,
    // when you call blit_byte_ascending,
    // then a single byte should be transferred, leaving the BlitContext in a state where subsequent
    // calls to blit_byte_ascending will seamlessly continue the blit operation.
    #[test]
    fn blit_byte_ascending_tracks_source_bytes() {
        let src: [u8; 1] = [0x21];
        let mut dst: [u8; 1] = [0x00];

        let mut bc = BlitContext::new(&src, 1, &mut dst, 1);
        bc.s_data = 0x45;
        bc.s_shift = 4;

        blit_byte_ascending(&mut bc);

        assert_eq!(bc.d_bits[0], 0x52);
        assert_eq!(bc.s_data, 0x21);
        assert_eq!(bc.s_ptr, 1);
        assert_eq!(bc.d_ptr, 1);
    }

    // Given that you have a properly initialized BlitContext structure,
    // when you call blit_byte_descending,
    // then a single byte should be transferred, leaving the BlitContext in a state where subsequent
    // calls to blit_byte_descending will seamlessly continue the blit operation.
    #[test]
    fn blit_byte_descending_tracks_source_bytes() {
        let src: [u8; 1] = [0x21];
        let mut dst: [u8; 1] = [0x00];

        let mut bc = BlitContext::new(&src, 1, &mut dst, 1);
        bc.s_data = 0x45;
        bc.s_shift = 4;

        blit_byte_descending(&mut bc);

        assert_eq!(bc.d_bits[0], 0x14);
        assert_eq!(bc.s_data, 0x21);
        assert_eq!(bc.s_ptr, !0);
        assert_eq!(bc.d_ptr, !0);
    }
}

// This abstraction level isn't much higher than working with individual bytes.
// However, it is convenient for expressing boolean operations to perform on entire
// vectors of bytes.
mod line_level {
    use crate::{BlitOp, BlitContext};
    use crate::{
        blit_line_ascending, blit_line_descending,
    };

    // Given that you have a properly initialized BlitContext structure,
    // when you try to blit a zero-width line,
    // then the context should remain unaltered.
    #[test]
    fn zero_width_blit_line_is_nop() {
        let src: [u8; 2] = [0x11, 0x22];
        let mut dst: [u8; 2] = [0, 0];

        let mut bc = BlitContext::new(&src, 2, &mut dst, 2);
        bc.operation = BlitOp::S;

        blit_line_ascending(&mut bc, 0);
        assert_eq!(bc.d_bits, &[0, 0]);
        assert_eq!(bc.d_ptr, 0);
        assert_eq!(bc.s_ptr, 0);
        assert_eq!(bc.s_data, 0);

        blit_line_descending(&mut bc, 0);
        assert_eq!(bc.d_bits, &[0, 0]);
        assert_eq!(bc.d_ptr, 0);
        assert_eq!(bc.s_ptr, 0);
        assert_eq!(bc.s_data, 0);
    }

    // Given that we want to blit a one-byte raster line,
    // when performing the blit,
    // then the first- and last-column masks both apply to the source data fetched.
    // WHY: The source byte fetched is simultaneously the first and last column in the raster line.
    #[test]
    fn blit_one_byte_line() {
        let src: [u8; 2] = [0xFF, 0xFF];
        let mut dst: [u8; 2] = [0, 0];

        let mut bc = BlitContext::new(&src, 2, &mut dst, 2);
        bc.s_fc_mask = 0x3F;
        bc.s_lc_mask = 0xFC;
        bc.operation = BlitOp::S;

        blit_line_ascending(&mut bc, 1);
        assert_eq!(bc.d_bits, &[0x3C, 0]);
        assert_eq!(bc.d_ptr, 1);
        assert_eq!(bc.s_ptr, 1);
        assert_eq!(bc.s_data, 0x3C);

        // Remember pointers already point at offset 1.
        blit_line_descending(&mut bc, 1);
        assert_eq!(bc.d_bits, &[0x3C, 0x3C]);
        assert_eq!(bc.d_ptr, 0);
        assert_eq!(bc.s_ptr, 0);
        assert_eq!(bc.s_data, 0x3C);
    }

    // Given that we want to blit a two-byte or wider raster line,
    // when performing the blit,
    // then the first- and last-column masks are separated,
    // and also bits shifted beyond a byte boundary are accounted for in subsequent bytes.
    #[test]
    fn blit_three_byte_line() {
        // Ascending case
        {
            let src: [u8; 4] = [0x12, 0x34, 0x56, 0x78];
            let mut dst: [u8; 4] = [0; 4];

            let mut bc = BlitContext::new(&src, 4, &mut dst, 4);
            bc.s_shift = 4;
            bc.s_fc_mask = 0x0F;
            bc.s_lc_mask = 0xF0;

            blit_line_ascending(&mut bc, 3);
            assert_eq!(bc.d_bits, &[0x00, 0x23, 0x45, 0x00]);
            assert_eq!(bc.d_ptr, 3);
            assert_eq!(bc.s_ptr, 3);
            assert_eq!(bc.s_data, 0x50);
        }

        // Descending case
        {
            let src: [u8; 4] = [0x12, 0x34, 0x56, 0x78];
            let mut dst: [u8; 4] = [0; 4];

            let mut bc = BlitContext::new(&src, 4, &mut dst, 4);
            bc.s_ptr = 3;
            bc.s_shift = 4;
            bc.s_fc_mask = 0xF0;
            bc.s_lc_mask = 0x0F;
            bc.d_ptr = 2;

            blit_line_descending(&mut bc, 3);
            assert_eq!(bc.d_bits, &[0x45, 0x67, 0x00, 0x00]);
            assert_eq!(bc.d_ptr, -1isize as usize);
            assert_eq!(bc.s_ptr, 0);
            assert_eq!(bc.s_data, 0x04);
        }
    }
}

// The highest level interface to the bit-blit library is the blit_rect function.
// Through this function, lower-level details like which operating mode are tucked away
// and dealt with for you.
mod rectangle_level {
    use crate::{BlitOp, BlitContext, blit_rect};

    // Given a source bitmap 8 pixels by 1 pixels,
    // and a destination bitmap also 8 pixels by 1 pixels,
    // when performing an unshifted blit from (0,0)-(8,1) into the destination at (0,0),
    // then we expect only one byte to be considered since the blit width is at most one.
    #[test]
    fn blit_rect_from_aligned_source_to_destination() {
        let src: [u8; 1] = [0x12];
        let mut dst: [u8; 1] = [0];

        let mut bc = BlitContext::new(&src, 1, &mut dst, 1);
        blit_rect(&mut bc, 0, 0, 8, 1, 0, 0, BlitOp::Or);

        assert_eq!(bc.d_bits[0], 0x12);
    }

    // Given a source bitmap 8x1 pixels,
    // and a destination bitmap 16x1 pixels,
    // when performing a shifted blit from (0,0)-(8,1) into the destination at (1,0),
    // then we expect a blit width of two.
    //
    // For this reason, we must also ensure our source bitmap is at least of length two bytes,
    // or we risk bounds check panics when performing the blit.
    //
    // KNOWN BUG: The blit_rect function may not yield the correct results if the source rectangle
    // width is smaller than the destination rectangle width *and* a shift needs to be applied.
    //
    // Although the source rectangle is (0,0)-(8,1), which should yield a source blit width of one
    // byte, the destination is placed one bit to the right, which means the destination width is
    // *two* bytes (bits 6-0 of the first, and bit 7 of the second).  Because the blit always uses
    // the larger of the two widths, the first- and last-column masks no longer align correctly
    // with the source rectangle.
    //
    // Therefore, two blits are required.  First, a temporary rectangle is constructed, into which
    // the source is copied using the BlitOp::S operator.  This performs the desired shift, but
    // note that there will almost certainly be some garbage to the right or left of the desired
    // destination rectangle as a result of the misaligned column masks.
    //
    // Once the "shift" has been performed, we copy only the desired destination rectangle from the
    // temporary rectangle to its final destination.  Since this does not perform any shifting, the
    // masks are guaranteed to align, and only those bits that are of interest will be affected.
    #[test]
    fn blit_rect_from_shifted_source_to_destination() {
        let src: [u8; 2] = [0x13, 0x55];

        // It would be great if we could just execute the following code:
        //
        // let mut bc = BlitContext::new(&src, 2, &mut dst, 2);
        // blit_rect(&mut bc, 0, 0, 8, 1, 1, 0, BlitOp::Or);
        // 
        // However, I can't figure out a way to make this case work.  Thankfully, there is a
        // work-around.
        //
        // First align the source bitmap in a temporary buffer so that any future blits will all
        // have the same width for both source and destination rectangles.
        let mut tmp: [u8; 2] = [0, 0];
        let mut bc = BlitContext::new(&src, 2, &mut tmp, 2);
        blit_rect(&mut bc, 0, 0, 8, 1, 1, 0, BlitOp::S);

        // Once that's done, NOW we can perform the desired blit.
        let mut dst: [u8; 2] = [0, 0];
        bc.s_bits = &(*bc.d_bits); // We can't use &tmp here because it's mutably borrowed.
        bc.d_bits = &mut dst;
        blit_rect(&mut bc, 1, 0, 9, 1, 1, 0, BlitOp::Or);

        // Now we have our correct and desired results.
        assert_eq!(bc.d_bits, [0b00001001, 0b10000000]);
    }

    // Going in the opposite direction,
    // given a source bitmap that is 16x1 pixels,
    // and a destination bitmap that is (ultimately) only 8x1,
    // when performing a shifted blit from (1,0)-(9,1) into the destination at (0,0),
    // then we still expect a blit width of two, even if the result is only a single byte wide.
    #[test]
    fn blit_rect_from_shifted_source_to_destination_backwards() {
        let src: [u8; 2] = [0x13, 0x55];
        let mut dst: [u8; 2] = [0, 0];

        let mut bc = BlitContext::new(&src, 2, &mut dst, 2);
        blit_rect(&mut bc, 1, 0, 9, 1, 0, 0, BlitOp::Or);
        assert_eq!(bc.d_bits, [0b00100110, 0b00000000]);
    }

    // While debugging issues with blit_rect_from_shifted_source_to_destination_backwards above,
    // I needed to inspect and confirm certain BlitContext values.  I refactored the code to grant
    // me access to the state of a BlitContext just before the blit_rect actually commences.
    //
    // This unit test somewhat exposes an internal design decision; thus, if this test starts
    // failing as a result of future maintenance, it is explicitly a candidate for replacement with
    // a more modern test, or removal all-together.
    //
    // Given a source rectangle that isn't byte-aligned,
    // when we call blit_rect() with a certain set of arguments
    // then we expect to see first and last column masks correctly assigned, and correct source and
    // destination pointers.
    #[test]
    fn prepare_blit_rect_from_shifted_source_to_destination_backwards() {
        let src: [u8; 2] = [0x13, 0x55];
        let mut dst: [u8; 2] = [0, 0];

        let mut bc = BlitContext::new(&src, 2, &mut dst, 2);
        let _ = crate::prepare_blit_rect(&mut bc, 1, 0, 9, 1, 0, 0, BlitOp::Or);
        assert_eq!(bc.d_ptr, 1);
        assert_eq!(bc.s_ptr, 1);
        assert_eq!(bc.s_fc_mask, 0x80);
        assert_eq!(bc.s_lc_mask, 0x7F);
    }

    // Illustrate how to draw a MacOS-style mouse pointer on a desktop background.
    #[test]
    fn blit_mouse_cursor() {
        // Let's start with a 50% grey stippled background, just like classic MacOS or GEOS.
        let mut hypothetical_desktop: [u8; 128] = [
            0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b01010101, 0b01010101, 0b01010101, 0b01010101,
        ];

        // Now, let's draw the mouse's white outline.
        //
        // Although the mouse is only 16x16, we still define a spare byte column on the right to
        // facilitate support for shifted blits.  Otherwise, blitting will result in slice
        // bounds-check issues when blitting into destination rectangles that are 3-bytes wide.
        let mouse_mask: [u8; 48] = [
            0b11000000, 0b00000000, 0b00000000,
            0b11110000, 0b00000000, 0b00000000,
            0b01111100, 0b00000000, 0b00000000,
            0b01111111, 0b00000000, 0b00000000,
            0b00111111, 0b11000000, 0b00000000,
            0b00111111, 0b11100000, 0b00000000,
            0b00011111, 0b11000000, 0b00000000,
            0b00011111, 0b11000000, 0b00000000,
            0b00001111, 0b11100000, 0b00000000,
            0b00001111, 0b11110000, 0b00000000,
            0b00000100, 0b11111000, 0b00000000,
            0b00000000, 0b01111000, 0b00000000,
            0b00000000, 0b00110000, 0b00000000,
            0b00000000, 0b00000000, 0b00000000,
            0b00000000, 0b00000000, 0b00000000,
            0b00000000, 0b00000000, 0b00000000,
        ];

        let mut bc = BlitContext::new(&mouse_mask, 3, &mut hypothetical_desktop, 4);
        blit_rect(&mut bc, 0, 0, 16, 16, 10, 10, BlitOp::Or);

        // Next, we draw the body of the mouse.
        let mouse_body: [u8; 48] = [
            0b00000000, 0b00000000, 0b00000000,
            0b01000000, 0b00000000, 0b00000000,
            0b00110000, 0b00000000, 0b00000000,
            0b00111100, 0b00000000, 0b00000000,
            0b00011111, 0b00000000, 0b00000000,
            0b00011111, 0b11000000, 0b00000000,
            0b00001111, 0b00000000, 0b00000000,
            0b00001111, 0b10000000, 0b00000000,
            0b00000101, 0b11000000, 0b00000000,
            0b00000100, 0b11100000, 0b00000000,
            0b00000000, 0b01110000, 0b00000000,
            0b00000000, 0b00110000, 0b00000000,
            0b00000000, 0b00000000, 0b00000000,
            0b00000000, 0b00000000, 0b00000000,
            0b00000000, 0b00000000, 0b00000000,
            0b00000000, 0b00000000, 0b00000000,
        ];

        bc.s_bits = &mouse_body;
        blit_rect(&mut bc, 0, 0, 16, 16, 10, 10, BlitOp::DandNotS);

        // The resulting desktop should look like the mouse pointer is somewhere in the center of
        // the 32x32 pixel matrix.
        let expected_desktop: [u8; 128] = [
            0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b10101010, 0b10111010, 0b10101010, 0b10101010,
            0b01010101, 0b01101101, 0b01010101, 0b01010101,
            0b10101010, 0b10110011, 0b10101010, 0b10101010,
            0b01010101, 0b01010000, 0b11010101, 0b01010101,
            0b10101010, 0b10101000, 0b00111010, 0b10101010,
            0b01010101, 0b01011000, 0b00001101, 0b01010101,
            0b10101010, 0b10101100, 0b00111010, 0b10101010,
            0b01010101, 0b01010100, 0b00010101, 0b01010101,
            0b10101010, 0b10101010, 0b10001010, 0b10101010,
            0b01010101, 0b01010110, 0b11000101, 0b01010101,
            0b10101010, 0b10101011, 0b10100010, 0b10101010,
            0b01010101, 0b01010101, 0b01010011, 0b01010101,
            0b10101010, 0b10101010, 0b10101110, 0b10101010,
            0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b01010101, 0b01010101, 0b01010101, 0b01010101,
        ];

        // For pretty-printing purposes, let's dump the XOR between the hypothetical and expected
        // desktop patterns.  This should be a 32x32 matrix of dots (.) if all is well.
        for i in 0..128 {
            let h = bc.d_bits[i];
            let e = expected_desktop[i];
            let mut x = h ^ e;

            for _ in 0..8 {
                if (x & 0x80) != 0 { eprint!("*"); } else { eprint!("."); };
                x <<= 1;
            }
            if (i & 3) == 3 { eprintln!(""); }
        }

        assert_eq!(bc.d_bits, &expected_desktop);
    }
}

