//! All things having to do with SDL input and output.

use sdl2;


/// A convenient anchor for SDL-related data structures.
pub struct SdlState {
    pub context: sdl2::Sdl,
    pub canvas: sdl2::render::Canvas<sdl2::video::Window>,
    pub width: u32,
    pub height: u32,
}

impl SdlState {
    /// Opens a fixed-sized SDL window of the given title, width, and height.
    pub fn new(title: &str, width: u32, height: u32) -> Self {
        let sdl = sdl2::init().unwrap();
        let video_subsystem = sdl.video().unwrap();
        let window = sdl2::video::WindowBuilder::new(&video_subsystem, title, width, height)
            .build()
            .unwrap();
        let canvas = window.into_canvas().build().unwrap();

        Self {
            context: sdl,
            canvas,
            width,
            height,
        }
    }

    /// Prepares the SDL surface for repainting.  Invokes the supplied closure.  Then, concludes
    /// the painting process and commits the changes to the display.
    pub fn paint_with<PainterFn>(&mut self, f: PainterFn)
    where
        PainterFn: Fn(&mut TextureContext) -> (),
    {
        let tc = self.canvas.texture_creator();
        let t = tc.create_texture(
            Some(sdl2::pixels::PixelFormatEnum::RGBA8888),
            sdl2::render::TextureAccess::Streaming,
            self.width,
            self.height,
        )
        .unwrap();
        let mut ctx = TextureContext::new(t);
        ctx.texture.set_blend_mode(sdl2::render::BlendMode::None);

        f(&mut ctx);

        self.canvas.copy(&ctx.texture, None, None).unwrap();
        self.canvas.present();
    }
}


/// When painting, this handle attempts to work around the borrow checker's steadfast refusal to
/// let me reference a texture inside the SdlState structure.
/// 
/// This structure provides a convenient interface to the texture onto which you're painting.
pub struct TextureContext<'a> {
    pub texture: sdl2::render::Texture<'a>,
}

impl<'a> TextureContext<'a> {
    pub fn new(texture: sdl2::render::Texture<'a>) -> Self {
        Self {
            texture,
        }
    }

    /// Pastes a bitmap "stamp" onto the SDL surface.  Source bits are assumed to be packed in
    /// big-endian order.  In other words, the left-most bit of the left-most byte is taken to be
    /// the first pixel of a raster row.  Two adjacent bytes in the bitmap will have their bits
    /// arranged as follows:
    ///
    /// | Pixel | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | ... |
    /// |-------|---|---|---|---|---|---|---|---|---|---|----|----|----|----|----|----|-----|
    /// | Byte  | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 | 1  | 1  | 1  | 1  | 1  | 1  | ... |
    /// | Bit   | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 | 7 | 6 | 5  | 4  | 3  | 2  | 1  | 0  | ... |
    ///
    /// This arrangement is ideal for statically defined bitmapped images and icons placed directly
    /// in the source code.
    ///
    /// Currently, 1-bits in the bitmap correspond to *white* pixels in the output texture,
    /// while 0-bits correspond to *black* pixels.
    ///
    /// **Please note:** no clipping is performed.  You need to know what you're pasting and where,
    /// or you will risk undefined behavior.  Since this is a Rust app, that probably means either
    /// a core- or panic!-dump, depending on how the SDL bindings work.

    pub fn paste_stamp_be(
        &mut self,
        src_pt: (usize, usize),
        src_dim: (usize, usize),
        span: usize,
        dst_pt: (usize, usize),
        bitmap: &[u8],
    ) {
        let (src_x, src_y) = src_pt;
        let (width, height) = src_dim;
        let (dst_x, dst_y) = dst_pt;
        let pixel_size = 4; // because of RGBA8888; TODO: query size from the enum.

        self.texture.with_lock(None, |pixels: &mut [u8], stride| {
            for i in 0..height {
                let y = i + src_y;
                let mut o = (i + dst_y) * stride + (dst_x * pixel_size);
                for x in src_x..src_x + width {
                    let bit_pos = 7 - (x & 7);
                    let byte_pos = x >> 3;
                    let bit = bitmap[y * span + byte_pos] & (1 << bit_pos);
                    pixels[o + 0] = 0xFF;
                    if bit != 0 {
                        pixels[o + 1] = 0xFF;
                        pixels[o + 2] = 0xFF;
                        pixels[o + 3] = 0xFF;
                    } else {
                        pixels[o + 1] = 0x00;
                        pixels[o + 2] = 0x00;
                        pixels[o + 3] = 0x00;
                    }
                    o = o + pixel_size;
                }
            }
        })
        .unwrap();
    }
}

