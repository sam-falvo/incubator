// Working at the level of individual bytes,
// these operations are the lowest level of abstraction for blitting something.
mod byte_level {
    use crate::{BlitOp, BlitContext};
    use crate::{
        blit_byte_ascending, blit_byte_descending,
    };

    // Given that you have a properly initialized BlitContext structure,
    // when you call blit_byte_ascending,
    // then a single byte should be transferred, leaving the BlitContext in a state where subsequent
    // calls to blit_byte_ascending will seamlessly continue the blit operation.
    #[test]
    fn blit_byte_ascending_tracks_source_bytes() {
        let src: [u8; 1] = [0x21];
        let mut dst: [u8; 1] = [0x00];

        let mut bc = BlitContext::new(&src, &mut dst);
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

        let mut bc = BlitContext::new(&src, &mut dst);
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
// However, it is convenient for expressing boolean operations to perform on vectors of bytes.
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

        let mut bc = BlitContext::new(&src, &mut dst);
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

        let mut bc = BlitContext::new(&src, &mut dst);
        bc.s_fc_mask = 0x3F;
        bc.s_lc_mask = 0xFC;
        bc.operation = BlitOp::S;

        blit_line_ascending(&mut bc, 1);
        assert_eq!(bc.d_bits, &[0x3C, 0]);
        assert_eq!(bc.d_ptr, 1);
        assert_eq!(bc.s_ptr, 1);
        assert_eq!(bc.s_data, 0xFF);

        // Remember pointers already point at offset 1.
        blit_line_descending(&mut bc, 1);
        assert_eq!(bc.d_bits, &[0x3C, 0x3C]);
        assert_eq!(bc.d_ptr, 0);
        assert_eq!(bc.s_ptr, 0);
        assert_eq!(bc.s_data, 0xFF);
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

            let mut bc = BlitContext::new(&src, &mut dst);
            bc.s_shift = 4;
            bc.s_fc_mask = 0x0F;
            bc.s_lc_mask = 0xF0;

            blit_line_ascending(&mut bc, 3);
            assert_eq!(bc.d_bits, &[0x01, 0x23, 0x40, 0x00]);
            assert_eq!(bc.d_ptr, 3);
            assert_eq!(bc.s_ptr, 3);
            assert_eq!(bc.s_data, 0x56);
        }

        // Descending case
        {
            let src: [u8; 4] = [0x12, 0x34, 0x56, 0x78];
            let mut dst: [u8; 4] = [0; 4];

            let mut bc = BlitContext::new(&src, &mut dst);
            bc.s_ptr = 3;
            bc.s_shift = 4;
            bc.s_fc_mask = 0xF0;
            bc.s_lc_mask = 0x0F;
            bc.d_ptr = 2;

            blit_line_descending(&mut bc, 3);
            assert_eq!(bc.d_bits, &[0x05, 0x67, 0x80, 0x00]);
            assert_eq!(bc.d_ptr, -1isize as usize);
            assert_eq!(bc.s_ptr, 0);
            assert_eq!(bc.s_data, 0x34);
        }
    }
}

