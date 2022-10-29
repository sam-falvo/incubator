use stencil::utils::draw_desktop;
use stencil::stencil::Stencil;
use stencil::types::{Point, Rect};

/// The root controller shows the desktop background, and keeps track of the mouse pointer.
/// The root presenter is given by the root environment.
/// There is no defined root model per se; it is whatever your app needs it to be.
pub struct ToyState {
    mouse: MouseState,
    subcomponents: Vec<PushButton<bool>>,
}

pub trait RootController: MouseEventSink + Controller { }

impl RootController for ToyState { }

impl MouseEventSink for ToyState {
    fn pointer_moved(&mut self, p: Point)-> RoseRequest  {
        self.mouse.xy = p;
        RoseRequest::None
    }

    fn button_down(&mut self)-> RoseRequest  {
        self.mouse.pressed = true;
        RoseRequest::None
    }

    fn button_up(&mut self) -> RoseRequest {
        self.mouse.pressed = false;
        RoseRequest::None
    }
}

impl Controller for ToyState {
    fn draw(&mut self, desktop: &mut Stencil) {
        draw_desktop(desktop);

        for pb in &mut self.subcomponents {
            pb.draw(desktop);
        }
    }
}

impl ToyState {
    pub fn new() -> Self {
        let mut s = Self {
            mouse: MouseState::new(),
            subcomponents: Vec::new(),
        };

        s.subcomponents.push(PushButton::new(((8, 8), (72, 28)), false));
        s.subcomponents.push(PushButton::new(((80, 8), (144, 28)), true));

        s
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
    fn pointer_moved(&mut self, to: Point) -> RoseRequest;
    fn button_up(&mut self) -> RoseRequest;
    fn button_down(&mut self) -> RoseRequest;
}

/// Initialize application state and render it for the first time.
/// Answer with a global event handler.
pub fn init_root() -> Box<dyn RootController> {
    Box::new(ToyState::new())
}

pub enum RoseRequest {
    None,
    Quit,
    RepaintAll,
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
}

impl<A: BoolViewAbstraction> PushButton<A> {
    fn new(area: Rect, default_state: A) -> Self {
        Self {
            view: PushButtonView::new(area),
            abstraction: default_state,
        }
    }
}
