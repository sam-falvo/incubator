use std::rc::Rc;
use std::cell::RefCell;

use stencil::utils::draw_desktop;
use stencil::stencil::Stencil;
use stencil::types::{Point, Rect};

pub trait RootController: MouseEventSink + AppController + View { }

pub trait View {
    fn draw(&self, med: &mut dyn Mediator);
    fn encloses_point(&self, p: Point) -> bool;
}

pub trait AppController {
    fn request_quit(&self) -> bool;
}

pub trait Mediator {
    fn repaint_all(&mut self);
    fn quit(&mut self);
    fn borrow_mut_desktop(&mut self) -> &mut Stencil;
}

/// Mouse-related event sink.
///
/// Typically implemented by presenters, but this can also be implemented by controllers as well if
/// the presenter just funnels events directly through to the controller.
pub trait MouseEventSink {
    fn pointer_moved(&mut self, med: &mut dyn Mediator, to: Point);
    fn button_up(&mut self, med: &mut dyn Mediator);
    fn button_down(&mut self, med: &mut dyn Mediator);
    fn enter(&mut self, med: &mut dyn Mediator, at: Point);
    fn leave(&mut self, med: &mut dyn Mediator);
}

// ------------------------------------------------------------------------

pub fn init_root(med: &mut dyn Mediator) -> Box<dyn RootController> {
    let toybox = Box::new(ToyState::new());

    let desktop = med.borrow_mut_desktop();
    draw_desktop(desktop);
    (*toybox).draw(med);

    toybox
}

pub struct ToyState {
    gadgets: Vec<Rc<RefCell<dyn Gadget>>>,
    hot: Option<usize>,
}

impl RootController for ToyState { }

impl AppController for ToyState {
    fn request_quit(&self) -> bool {
        // We have no reason to deny quitting, so yes.
        true
    }
}

impl ToyState {
    pub fn new() -> Self {
        Self {
            gadgets: vec![
                Rc::new(RefCell::new(PushButtonGadget::new(((8,8), (72, 28)), false))),
                Rc::new(RefCell::new(PushButtonGadget::new(((80,8), (144, 28)), true))),
                Rc::new(RefCell::new(PushButtonGadget::new(((152,8), (216, 28)), true))),
            ],
            hot: None,
        }
    }

    fn select_next_hot_gadget(&mut self, p: Point, med: &mut dyn Mediator) {
        self.hot = self.gadgets.iter().position(|g| {
            g.borrow().as_view().encloses_point(p)
        });

        if let Some(idx) = self.hot {
            self.gadgets[idx].borrow_mut().as_mut_controller().enter(med, p);
        }
    }

    fn check_events(&self, med: &mut dyn Mediator) {
        if self.gadgets[0].borrow_mut().get_events() != 0 {
            eprintln!("Event on button 1");
        }

        if self.gadgets[0].borrow_mut().get_events() != 0 {
            eprintln!("UNEXPECTED Event on button 1");
        }

        if self.gadgets[1].borrow_mut().get_events() != 0 {
            eprintln!("Event on button 2");
        }

        if self.gadgets[2].borrow_mut().get_events() != 0 {
            eprintln!("Event on button 3");
            med.quit();
        }
    }
}

impl View for ToyState {
    fn draw(&self, med: &mut dyn Mediator) {
        for g in &self.gadgets {
            g.borrow().as_view().draw(med);
        }
    }

    fn encloses_point(&self, _: Point) -> bool {
        // We're full-screen, so yes.
        true
    }
}

impl MouseEventSink for ToyState {
    fn pointer_moved(&mut self, med: &mut dyn Mediator, p: Point) {
        match self.hot {
            None => self.select_next_hot_gadget(p, med),
            Some(idx) => {
                let g = &self.gadgets[idx];
                if g.borrow().as_view().encloses_point(p) {
                    g.borrow_mut().as_mut_controller().pointer_moved(med, p);
                } else {
                    g.borrow_mut().as_mut_controller().leave(med);
                    self.select_next_hot_gadget(p, med);
                }
                self.check_events(med);
            }
        }
    }

    fn button_down(&mut self, med: &mut dyn Mediator) {
        if let Some(idx) = self.hot {
            self.gadgets[idx].borrow_mut().as_mut_controller().button_down(med);
        }
        self.check_events(med);
    }

    fn button_up(&mut self, med: &mut dyn Mediator) {
        if let Some(idx) = self.hot {
            self.gadgets[idx].borrow_mut().as_mut_controller().button_up(med);
        }
        self.check_events(med);
    }

    fn enter(&mut self, _: &mut dyn Mediator, _: Point) {
        // Nothing to do
    }

    fn leave(&mut self, _: &mut dyn Mediator) {
        // Nothing to do
    }
}


// ------------------------------------------------------------------------
// Boolean gadgets

use std::cell::Cell;

use stencil::stencil::Draw;
use stencil::utils::{LINE_BLACK, LINE_WHITE, BLACK_PATTERN, WHITE_PATTERN};

pub type EventMask = u16;

pub trait Gadget {
    fn as_view(&self) -> &dyn View;
    fn as_mut_controller(&mut self) -> &mut dyn MouseEventSink;
    fn get_events(&mut self) -> EventMask;
}

pub struct PushButtonGadget {
    area: Rect,
    is_pressed: bool,
    needs_repaint: Cell<bool>,
    events: u16,
}

const PBG_EVENTF_CLICKED: u16 = 0x0001;

impl PushButtonGadget {
    pub fn new(area: Rect, is_pressed: bool) -> Self {
        Self {
            area,
            is_pressed,
            needs_repaint: Cell::new(true),
            events: 0,
        }
    }

    fn repaint_needed(&mut self, med: &mut dyn Mediator) {
        med.repaint_all();
        self.needs_repaint.set(true);
    }

    fn button_was_clicked(&mut self) {
        self.events |= PBG_EVENTF_CLICKED;
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
    fn draw(&self, med: &mut dyn Mediator) {
        let s = med.borrow_mut_desktop();

        if self.is_pressed {
            self.draw_pb_pressed(s);
        } else {
            self.draw_pb_unpressed(s);
        }
        med.repaint_all();
    }

    fn encloses_point(&self, p: Point) -> bool {
        let (x, y) = p;
        let ((left, top), (right, bottom)) = self.area;

        (left <= x) && (x < right) && (top <= y) && (y < bottom)
    }
}

impl MouseEventSink for PushButtonGadget {
    fn pointer_moved(&mut self, _: &mut dyn Mediator, _: Point) {
        // Nothing to do
    }

    fn button_down(&mut self, med: &mut dyn Mediator) {
        self.is_pressed = !self.is_pressed;
        self.draw(med);
    }

    fn button_up(&mut self, med: &mut dyn Mediator) {
        self.is_pressed = !self.is_pressed;
        self.draw(med);
        self.button_was_clicked();
    }

    fn enter(&mut self, _med: &mut dyn Mediator, _: Point) {
        // Nothing to do
    }

    fn leave(&mut self, _med: &mut dyn Mediator) {
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

    fn get_events(&mut self) -> EventMask {
        let e = self.events;
        self.events = 0;
        e
    }
}

