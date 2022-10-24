use stencil::stencil::Draw;
use stencil::utils::draw_desktop;

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

impl Initializable for Reader {
    fn init(&mut self, desktop: &mut impl Draw) {
        draw_desktop(desktop);
    }
}

