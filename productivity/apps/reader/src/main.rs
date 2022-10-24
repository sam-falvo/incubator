extern crate sdl2;
extern crate sdlstate;

const W: Dimension = 320;
const H: Dimension = 200;

use sdl2::event::{Event, WindowEvent};
use sdlstate::SdlState;
use stencil::types::Dimension;
use stencil::stencil::{Stencil, Draw};
use app::{Reader, Initializable};

fn main() {
    // Create the SDL bindings.
    let mut sdl = SdlState::new("File Reader", W as u32, H as u32);
    let mut event_pump = sdl.context.event_pump().expect("event pump issue");
    let mut event_iter = event_pump.wait_iter();

    let mut desktop = Stencil::new_with_dimensions(W, H);

    let mut app_state = Reader::new();
    app_state.init(&mut desktop);

    'main_event_loop: loop {
        for e in &mut event_iter {
            match e {
                Event::Quit { .. } => break 'main_event_loop,
                Event::Window { win_event: we, .. } => {
                    if we == WindowEvent::Exposed {
                        repaint(&mut desktop, &mut sdl)
                    }
                },
                _ => (),
            }
        }
    }
}

fn repaint(desktop: &mut Stencil, sdl: &mut SdlState) {
    let origin = (0, 0);
    let (w, h) = desktop.get_dimensions();
    let dimensions = (w as usize, h as usize);

    sdl.paint_with(|ctx| {
        ctx.paste_stamp_be(
            origin,
            dimensions,
            desktop.get_span(),
            origin,
            desktop.borrow_bits(),
        );
    });
}

mod app;

