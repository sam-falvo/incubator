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
    #[test]
    fn blit_rect_from_shifted_source_to_destination() {
        let src: [u8; 2] = [0x13, 0x55];
        let mut dst: [u8; 2] = [0, 0];

        let mut bc = BlitContext::new(&src, 2, &mut dst, 2);
        blit_rect(&mut bc, 0, 0, 8, 1, 1, 0, BlitOp::Or);

        assert_eq!(bc.d_bits[0], 0b00001001);
        assert_eq!(bc.d_bits[1], 0b10000000);
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

}

