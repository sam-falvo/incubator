use stencil::stencil::{Pattern, Draw};

pub struct Reader {
}

impl Reader {
    pub fn new() -> Self {
        Reader{}
    }
}

pub trait Initializable {
    fn init(&mut self, desktop: &mut impl Draw);
}

static DESKTOP_PATTERN: Pattern = [0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55];

impl Initializable for Reader {
    fn init(&mut self, desktop: &mut impl Draw) {
        let (width, height) = desktop.get_dimensions();

        desktop.filled_rectangle((0, 0), (width, height), &DESKTOP_PATTERN);
    }
}

