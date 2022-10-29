use std::rc::Rc;
use std::cell::RefCell;

use stencil::utils::draw_desktop;
use stencil::stencil::Stencil;
use stencil::types::{Dimensions, Point, Rect};

/// The root controller shows the desktop background, and keeps track of the mouse pointer.
/// The root presenter is given by the root environment.
/// There is no defined root model per se; it is whatever your app needs it to be.
pub struct ToyState {
    mouse: MouseState,
    subcontrols: Vec<Rc<RefCell<PushButton<bool>>>>,
    hot: Option<Rc<RefCell<PushButton<bool>>>>,
}

pub trait RootController: MouseEventSink + Controller + AppController { }

impl RootController for ToyState { }

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
        match self.hot.clone() {
            None => {
                self.hot = self.find_enclosing_control(p);
                if let Some(_) = self.hot {
                    eprintln!("New hotness");
                }
            },
            Some(h) => {
                if !h.borrow().has_point(p) {
                    self.hot = self.find_enclosing_control(p);
                    eprintln!("From old hotness, we get a new hotness.");
                } 
            },
        }
    }

    fn button_down(&mut self) {
        self.mouse.pressed = true;
    }

    fn button_up(&mut self) {
        self.mouse.pressed = false;
    }
}

impl Controller for ToyState {
    fn draw(&mut self, desktop: &mut Stencil) {
        draw_desktop(desktop);

        for sc_refcell in &self.subcontrols {
            sc_refcell.borrow_mut().draw(desktop);
        }
    }

    fn has_point(&self, _: Point) -> bool {
        // We're full-screen, so yes.
        true
    }
}

impl ToyState {
    pub fn new(_display_dimensions: Dimensions) -> Self {
        let mut s = Self {
            mouse: MouseState::new(),
            subcontrols: Vec::new(),
            hot: None,
        };

        s.subcontrols.push(Rc::new(RefCell::new(PushButton::new(((8, 8), (72, 28)), false))));
        s.subcontrols.push(Rc::new(RefCell::new(PushButton::new(((80, 8), (144, 28)), true))));

        s
    }

    fn find_enclosing_control(&self, p: Point) -> Option<Rc<RefCell<PushButton<bool>>>> {
        for subctl in &self.subcontrols {
            let sc = subctl.borrow();

            if sc.has_point(p) {
                return Some(subctl.clone());
            }
        }
        None
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
// Boolean gadget components

// // Presentation

use stencil::stencil::Draw;
use stencil::utils::{LINE_BLACK, LINE_WHITE, BLACK_PATTERN, WHITE_PATTERN};

struct PushButtonView {
    area: Rect,
}

impl PushButtonView {
    fn new(area: Rect) -> Self {
        Self {
            area,
        }
    }

    fn get_area(&self) -> Rect {
        self.area
    }

    fn draw_unpressed(&self, s: &mut Stencil) {
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

    fn draw_pressed(&self, s: &mut Stencil) {
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

// // Abstraction

pub trait BoolViewAbstraction {
    fn is_selected(&self) -> bool;
}

impl BoolViewAbstraction for bool {
    fn is_selected(&self) -> bool {
        *self == true
    }
}

// // Controller

pub trait Controller {
    fn draw(&mut self, s: &mut Stencil);
    fn has_point(&self, p: Point) -> bool;
}

struct PushButton<A: BoolViewAbstraction> {
    view: PushButtonView,
    abstraction: A,
}

impl<A: BoolViewAbstraction> Controller for PushButton<A> {
    fn draw(&mut self, s: &mut Stencil) {
        if self.abstraction.is_selected() {
            self.view.draw_pressed(s);
        } else {
            self.view.draw_unpressed(s);
        }
    }

    fn has_point(&self, p: Point) -> bool {
        is_point_within(p, self.view.get_area())
    }
}

fn is_point_within(p: Point, r: Rect) -> bool {
    let (x, y) = p;
    let ((left, top), (right, bottom)) = r;

    (left <= x) && (x < right) && (top <= y) && (y < bottom)
}

impl<A: BoolViewAbstraction> PushButton<A> {
    fn new(area: Rect, default_state: A) -> Self {
        Self {
            view: PushButtonView::new(area),
            abstraction: default_state,
        }
    }
}

// ------------------------------------------------------------------------
// Event Loop Control interface
//
// NOTE: This is NOT a Controller.

pub trait EventLoopControl {
    fn approve_quit(&mut self);
}

