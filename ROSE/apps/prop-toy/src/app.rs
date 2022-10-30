use stencil::utils::draw_desktop;
use stencil::stencil::Stencil;
use stencil::types::{Dimensions, Point, Rect};

pub struct ToyState {
    mouse: MouseState,
}

pub trait RootController: MouseEventSink + AppController + View { }

impl RootController for ToyState { }

pub trait View {
    fn draw(&mut self, stencil: &mut Stencil);
}

impl View for ToyState {
    fn draw(&mut self, s: &mut Stencil) {
        draw_desktop(s);
    }
}

pub trait AppController {
    fn request_quit(&self) -> bool;
}

impl AppController for ToyState {
    fn request_quit(&self) -> bool {
        true
    }
}

impl MouseEventSink for ToyState {
    fn pointer_moved(&mut self, p: Point) {
        self.mouse.xy = p;
    }

    fn button_down(&mut self) {
        self.mouse.pressed = true;
    }

    fn button_up(&mut self) {
        self.mouse.pressed = false;
    }
}

impl ToyState {
    pub fn new(_display_dimensions: Dimensions) -> Self {
        Self {
            mouse: MouseState::new(),
        }
    }
}

/// MouseState keeps track of the current mouse position and button state.
pub struct MouseState {
    /// The current mouse pointer location.
    xy: Point,

    /// True if the button is pressed.
    pressed: bool,
}

impl MouseState {
    pub fn new() -> Self {
        Self {
            xy: (0, 0),
            pressed: false,
        }
    }
}

/// Mouse-related event sink.
///
/// Typically implemented by presenters, but this can also be implemented by controllers as well if
/// the presenter just funnels events directly through to the controller.
pub trait MouseEventSink {
    fn pointer_moved(&mut self, to: Point);
    fn button_up(&mut self);
    fn button_down(&mut self);
}

/// Initialize application state and render it for the first time.
/// Answer with a global event handler.
pub fn init_root(display_dimensions: Dimensions) -> Box<dyn RootController> {
    Box::new(ToyState::new(display_dimensions))
}

// ------------------------------------------------------------------------
// Event Loop Control interface
//
// NOTE: This is NOT a Controller.

pub trait EventLoopControl {
    fn approve_quit(&mut self);
}

// ------------------------------------------------------------------------
// Boolean gadget components

