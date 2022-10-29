extern crate sdl2;
extern crate sdlstate;

const W: Dimension = 320;
const H: Dimension = 200;

use sdl2::event::{Event, WindowEvent};
use sdl2::mouse::MouseButton;
use sdlstate::SdlState;
use stencil::types::{Dimension, Unit};
use stencil::stencil::{Stencil, Draw};
use app::{init_root, RoseRequest};

fn main() {
    // Create the SDL bindings.
    let mut sdl = SdlState::new("Prop Toy", W as u32, H as u32);
    let mut event_pump = sdl.context.event_pump().expect("event pump issue");
    let mut event_iter = event_pump.wait_iter();

    let mut desktop = Stencil::new_with_dimensions(W, H);

    let mut root = init_root();
    root.draw(&mut desktop);
    'main_event_loop: loop {
        for e in &mut event_iter {
            let req = match e {
                Event::Quit { .. } => RoseRequest::Quit,
                Event::MouseMotion { x, y, .. } => root.pointer_moved((x as Unit, y as Unit)),
                Event::MouseButtonDown { mouse_btn: b, .. } if b == MouseButton::Left => root.button_down(),
                Event::MouseButtonUp { mouse_btn: b, .. } if b == MouseButton::Left => root.button_up(),
                Event::Window { win_event: we, .. } if we == WindowEvent::Exposed => RoseRequest::RepaintAll,
                _ => RoseRequest::None,
            };

            match req {
                RoseRequest::None => (),
                RoseRequest::RepaintAll => repaint(&mut desktop, &mut sdl),
                RoseRequest::Quit => break 'main_event_loop,
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

