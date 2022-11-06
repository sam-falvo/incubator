extern crate sdl2;
extern crate sdlstate;

const W: Dimension = 320;
const H: Dimension = 200;

use sdl2::event::{Event, WindowEvent};
use sdl2::mouse::MouseButton;
use sdlstate::SdlState;
use stencil::types::{Dimension, Unit};
use stencil::stencil::{Stencil, Draw};
use app::{init_root, Mediator};

fn main() {
    // Create the SDL bindings.
    let mut sdl = SdlState::new("Prop Toy", W as u32, H as u32);
    let mut event_pump = sdl.context.event_pump().expect("event pump issue");
    let mut event_iter = event_pump.wait_iter();

    let mut desktop = Stencil::new_with_dimensions(W, H);
    let mut top_mediator = TopMediator::new(&mut desktop);
    let mut app = init_root(&mut top_mediator);
    'main_event_loop: loop {
        for e in &mut event_iter {
            match e {
                Event::Quit { .. } => {
                    if app.request_quit() {
                        break 'main_event_loop;
                    }
                },
                Event::MouseMotion { x, y, .. } => {
                    app.pointer_moved(&mut top_mediator, (x as Unit, y as Unit));
                    top_mediator.try_redrawing(&mut sdl);
                }
                Event::MouseButtonDown { mouse_btn: b, .. } if b == MouseButton::Left => {
                    app.button_down(&mut top_mediator);
                    top_mediator.try_redrawing(&mut sdl);
                }
                Event::MouseButtonUp { mouse_btn: b, .. } if b == MouseButton::Left => {
                    app.button_up(&mut top_mediator);
                    top_mediator.try_redrawing(&mut sdl);
                }
                Event::Window { win_event: we, .. } if we == WindowEvent::Exposed => repaint(&mut top_mediator.desktop, &mut sdl),
                _ => (),
            }

            if top_mediator.quit_requested {
                if app.request_quit() {
                    break 'main_event_loop;
                } else {
                    top_mediator.clear_quit();
                }
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

struct TopMediator<'a> {
    desktop: &'a mut Stencil,
    quit_requested: bool,
    needs_repaint: bool,
}

impl<'a> TopMediator<'a> {
    fn new(desktop: &'a mut Stencil) -> Self {
        Self {
            desktop,
            quit_requested: false,
            needs_repaint: false,
        }
    }

    fn clear_quit(&mut self) {
        self.quit_requested = false;
    }

    fn try_redrawing(&mut self, sdl: &mut SdlState) {
        if self.needs_repaint {
            repaint(self.desktop, sdl);
            self.needs_repaint = false;
        }
    }
}

impl<'a> Mediator for TopMediator<'a> {
    fn repaint_all(&mut self) {
        self.needs_repaint = true;
    }

    fn quit(&mut self) {
        self.quit_requested = true;
    }

    fn borrow_mut_desktop(&mut self) -> &mut Stencil {
        self.desktop
    }
}

mod app;
mod proportional;
