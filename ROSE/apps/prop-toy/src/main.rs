extern crate sdl2;
extern crate sdlstate;

const W: Dimension = 320;
const H: Dimension = 200;

use sdl2::event::{Event, WindowEvent};
use sdl2::mouse::MouseButton;
use sdlstate::SdlState;
use stencil::types::{Dimension, Unit};
use stencil::stencil::{Stencil, Draw};
use app::{init_root, Mediator, RootController};

fn main() {
    // Create the SDL bindings.
    let mut sdl = SdlState::new("Prop Toy", W as u32, H as u32);
    let mut event_pump = sdl.context.event_pump().expect("event pump issue");
    let mut event_iter = event_pump.wait_iter();

    let mut desktop = Stencil::new_with_dimensions(W, H);

    let mut top_mediator = TopMediator::new();
    let mut root = init_root((W, H));
    root.draw(&mut desktop);
    'main_event_loop: loop {
        for e in &mut event_iter {
            match e {
                Event::Quit { .. } => {
                    if root.request_quit() {
                        break 'main_event_loop;
                    }
                },
                Event::MouseMotion { x, y, .. } => {
                    root.pointer_moved((x as Unit, y as Unit), &mut top_mediator);
                    top_mediator.try_redrawing(&mut *root, &mut desktop, &mut sdl);
                }
                Event::MouseButtonDown { mouse_btn: b, .. } if b == MouseButton::Left => {
                    root.button_down(&mut top_mediator);
                    top_mediator.try_redrawing(&mut *root, &mut desktop, &mut sdl);
                }
                Event::MouseButtonUp { mouse_btn: b, .. } if b == MouseButton::Left => {
                    root.button_up(&mut top_mediator);
                    top_mediator.try_redrawing(&mut *root, &mut desktop, &mut sdl);
                }
                Event::Window { win_event: we, .. } if we == WindowEvent::Exposed => repaint(&mut desktop, &mut sdl),
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

struct TopMediator {
    needs_repaint: bool,
}

impl TopMediator {
    fn new() -> Self {
        Self { needs_repaint: false, }
    }

    fn try_redrawing(&mut self, root: &mut dyn RootController, desktop: &mut Stencil, sdl: &mut SdlState) {
        if self.needs_repaint {
            root.draw(desktop);
            repaint(desktop, sdl);
            self.needs_repaint = false;
        }
    }
}

impl Mediator for TopMediator {
    fn repaint_needed(&mut self) {
        self.needs_repaint = true;
    }
}

mod app;

