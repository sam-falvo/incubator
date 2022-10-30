use std::rc::Rc;
use std::cell::RefCell;

use stencil::utils::draw_desktop;
use stencil::stencil::Stencil;
use stencil::types::{Dimensions, Point, Rect};

pub struct ToyState {
    mouse: MouseState,
    gadgets: Vec<Rc<RefCell<dyn Gadget>>>,
    hot: Option<Rc<RefCell<dyn Gadget>>>,
}

pub trait RootController: MouseEventSink + AppController + View { }

impl RootController for ToyState { }

pub trait View {
    fn draw(&self, stencil: &mut Stencil);
    fn encloses_point(&self, p: Point) -> bool;
}

impl View for ToyState {
    fn draw(&self, s: &mut Stencil) {
        draw_desktop(s);
        for g in &self.gadgets {
            g.borrow().as_view().draw(s);
        }
    }

    fn encloses_point(&self, _: Point) -> bool {
        // We're full-screen, so yes.
        true
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

        match &self.hot {
            None => {
                let candidate = self.gadgets.iter().find(|g| {
                    g.borrow().as_view().encloses_point(p)
                });

                if let Some(gg) = candidate {
                    self.hot = Some((*gg).clone());
                    gg.borrow_mut().as_mut_controller().enter(p);
                }
            },
            Some(h) => {
                if !h.borrow().as_view().encloses_point(p) {
                    h.borrow_mut().as_mut_controller().leave();
                    self.hot = None;

                    let candidate = self.gadgets.iter().find(|g| {
                        g.borrow().as_view().encloses_point(p)
                    });

                    if let Some(gg) = candidate {
                        self.hot = Some((*gg).clone());
                        gg.borrow_mut().as_mut_controller().enter(p);
                    }
                } else {
                    h.borrow_mut().as_mut_controller().pointer_moved(p);
                }
            },
        }
    }

    fn button_down(&mut self) {
        self.mouse.pressed = true;
        if let Some(gg) = &self.hot {
            gg.borrow_mut().as_mut_controller().button_down();
        }
    }

    fn button_up(&mut self) {
        self.mouse.pressed = false;
        if let Some(gg) = &self.hot {
            gg.borrow_mut().as_mut_controller().button_up();
        }
    }

    fn enter(&mut self, _: Point) {
        // Nothing to do
    }

    fn leave(&mut self) {
        // Nothing to do
    }
}

impl ToyState {
    pub fn new(_display_dimensions: Dimensions) -> Self {
        Self {
            mouse: MouseState::new(),
            gadgets: vec![
                Rc::new(RefCell::new(PushButtonGadget::new(((8,8), (72, 28)), false))),
            ],
            hot: None,
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
    fn enter(&mut self, at: Point);
    fn leave(&mut self);
}

/// Initialize application state and render it for the first time.
/// Answer with a global event handler.
pub fn init_root(display_dimensions: Dimensions) -> Box<dyn RootController> {
    Box::new(ToyState::new(display_dimensions))
}

// ------------------------------------------------------------------------
// Event Loop Control interface
//
// NOTE: This is NOT a Controller in an MVC or PAC sense.

pub trait EventLoopControl {
    fn approve_quit(&mut self);
}

// ------------------------------------------------------------------------
// Boolean gadgets

use stencil::stencil::Draw;
use stencil::utils::{LINE_BLACK, LINE_WHITE, BLACK_PATTERN, WHITE_PATTERN};

pub trait Gadget {
    fn as_view(&self) -> &dyn View;
    fn as_mut_controller(&mut self) -> &mut dyn MouseEventSink;
}

pub struct PushButtonGadget {
    area: Rect,
    is_pressed: bool,
}

impl PushButtonGadget {
    pub fn new(area: Rect, is_pressed: bool) -> Self {
        Self {
            area,
            is_pressed,
        }
    }

    fn draw_pb_unpressed(&self, s: &mut Stencil) {
        let ((btn_left, btn_top), (btn_right, btn_bottom)) = self.area;
        let border_left = btn_left;
        let border_top = btn_top;
        let border_right = btn_right - 1;
        let border_bottom = btn_bottom - 1;

        let r_shadow_left = btn_right - 1;
        let r_shadow_top = btn_top + 1;
        let r_shadow_bottom = btn_bottom;

        let b_shadow_left = btn_left + 1;
        let b_shadow_top = btn_bottom - 1;
        let b_shadow_right = btn_right;

        let subview_left = btn_left;
        let subview_top = btn_top;
        let subview_right = btn_right - 1;
        let subview_bottom = btn_bottom - 1;

        s.filled_rectangle((subview_left, subview_top), (subview_right, subview_bottom), &WHITE_PATTERN);
        s.framed_rectangle((border_left, border_top), (border_right, border_bottom), LINE_BLACK);
        s.horizontal_line((b_shadow_left, b_shadow_top), b_shadow_right, LINE_BLACK);
        s.vertical_line((r_shadow_left, r_shadow_top), r_shadow_bottom, LINE_BLACK);
    }

    fn draw_pb_pressed(&self, s: &mut Stencil) {
        let ((btn_left, btn_top), (btn_right, btn_bottom)) = self.area;
        let border_left = btn_left + 1;
        let border_top = btn_top + 1;
        let border_right = btn_right;
        let border_bottom = btn_bottom;

        let l_shadow_left = btn_left;
        let l_shadow_top = btn_top;
        let l_shadow_bottom = btn_bottom;

        let t_shadow_left = btn_left;
        let t_shadow_top = btn_top;
        let t_shadow_right = btn_right;

        let subview_left = btn_left + 2;
        let subview_top = btn_top + 2;
        let subview_right = btn_right - 1;
        let subview_bottom = btn_bottom - 1;

        s.filled_rectangle((subview_left, subview_top), (subview_right, subview_bottom), &BLACK_PATTERN);
        s.framed_rectangle((border_left, border_top), (border_right, border_bottom), LINE_WHITE);
        s.horizontal_line((t_shadow_left, t_shadow_top), t_shadow_right, LINE_WHITE);
        s.vertical_line((l_shadow_left, l_shadow_top), l_shadow_bottom, LINE_WHITE);
    }
}

impl View for PushButtonGadget {
    fn draw(&self, s: &mut Stencil) {
        if self.is_pressed {
            self.draw_pb_pressed(s);
        } else {
            self.draw_pb_unpressed(s);
        }
    }

    fn encloses_point(&self, p: Point) -> bool {
        let (x, y) = p;
        let ((left, top), (right, bottom)) = self.area;

        (left <= x) && (x < right) && (top <= y) && (y < bottom)
    }
}

impl MouseEventSink for PushButtonGadget {
    fn pointer_moved(&mut self, _to: Point) {
        eprintln!("P");
    }

    fn button_down(&mut self) {
        eprintln!("D");
    }

    fn button_up(&mut self) {
        eprintln!("U");
    }

    fn enter(&mut self, _: Point) {
        eprintln!("E");
        // Nothing to do
    }

    fn leave(&mut self) {
        eprintln!("L");
        // Nothing to do
    }
}

impl Gadget for PushButtonGadget {
    fn as_view(&self) -> &dyn View {
        self
    }

    fn as_mut_controller(&mut self) -> &mut dyn MouseEventSink {
        self
    }
}

