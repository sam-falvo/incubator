use stencil::events::{AppController, AppEventSink, MouseEventSink};
use stencil::mediator::Mediator;
use stencil::stencil::Draw;
use stencil::sysfont_bsw_9::SYSTEM_BITMAP_FONT;
use stencil::types::{Point, Rect, Unit};
use stencil::utils::{draw_desktop, draw_dialog_box};
use stencil::utils::{LINE_BLACK, WHITE_PATTERN};
use stencil::view::{rect_contains, View};

use stencil::gadgets::proportional::{PropGadgetEvent, PropGadgetView};
use crate::boolean::{PushButtonView, PushButtonEvent};

/// This is the main entry point to all ROSE applications.
///
/// It constructs a structure representing the total application state,
/// and returns it to the host environment's event loop.
///
/// It also renders the initial desktop image.
pub fn init_root(med: &mut dyn Mediator) -> Box<dyn AppController> {
    let mut toybox = Box::new(ToyBoxApp::new());
    toybox.draw(med);
    Box::new(ToyBoxApp::new())
}

/// The application state,
/// which directly or indirectly
/// includes all models and views on those models.
pub struct ToyBoxApp<'l, 'f> {
    dbox_area: Rect,
    mouse_pt: Point,
    selected: Selectable,
    hr_area: Rect,
    hr_cursor_left: Unit,
    hr_cursor_right: Unit,
    vr_area: Rect,
    vr_cursor_top: Unit,
    vr_cursor_bottom: Unit,
    hprop: PropGadgetView,
    vprop: PropGadgetView,
    xyprop: PropGadgetView,
    quit_btn: PushButtonView<'l, 'f>,
}

/// This toybox application
/// includes a number of custom gadgets
/// that don't exist in the standard library.
/// We need to keep track of
/// which parts of which gadgets
/// the user is currently interacting with.
/// This enumeration identifies those individual parts.
enum Selectable {
    None,
    LeftRulerKnob,
    RightRulerKnob,
    TopRulerKnob,
    BottomRulerKnob,
}

impl<'l, 'f> ToyBoxApp<'l, 'f> {
    /// Provides the application state with default values.
    pub fn new() -> Self {
        Self {
            dbox_area: ((8, 8), (240, 192)),
            mouse_pt: (0, 0),
            selected: Selectable::None,
            hr_area: ((16, 16), (202, 24)),
            hr_cursor_left: 16,
            hr_cursor_right: 201,
            vr_area: ((224, 46), (232, 184)),
            vr_cursor_top: 46,
            vr_cursor_bottom: 183,
            hprop: PropGadgetView::new(((16, 30), (202, 38))),
            vprop: PropGadgetView::new(((210, 46), (218, 184))),
            xyprop: PropGadgetView::new(((16, 46), (202, 184))),
            quit_btn: PushButtonView::new(
                ((248, 8), (312, 28)),
                "Quit",
                &SYSTEM_BITMAP_FONT,
            ),
        }
    }

    /// Draws the entire application state onto the screen.
    fn draw(&mut self, med: &mut dyn Mediator) {
        draw_desktop(med.borrow_mut_desktop());

        // Draw the quit button
        self.quit_btn.draw(med);

        // Draw the window in which our prop gadgets will sit.
        draw_dialog_box(med.borrow_mut_desktop(), self.dbox_area);

        // Draw the custom and standard gadgets.
        self.draw_rulers(med);
        self.draw_prop_gadgets(med);

        med.repaint_all();
    }

    fn draw_prop_gadgets(&mut self, med: &mut dyn Mediator) {
        self.hprop
            .set_knob(((self.hr_cursor_left, 30), (self.hr_cursor_right + 1, 38)));
        self.hprop.draw(med);

        self.vprop
            .set_knob(((210, self.vr_cursor_top), (218, self.vr_cursor_bottom + 1)));
        self.vprop.draw(med);

        self.xyprop.set_knob((
            (self.hr_cursor_left, self.vr_cursor_top),
            (self.hr_cursor_right + 1, self.vr_cursor_bottom + 1),
        ));
        self.xyprop.draw(med);
    }

    fn draw_rulers(&mut self, med: &mut dyn Mediator) {
        self.draw_h_ruler(med);
        self.draw_v_ruler(med);
    }

    fn draw_h_ruler(&mut self, med: &mut dyn Mediator) {
        let d = med.borrow_mut_desktop();
        let ((hr_left, hr_top), (hr_right, hr_bottom)) = self.hr_area;
        let hr_rule_y = (hr_top + hr_bottom) >> 1;
        let hr_cursor_left = self.hr_cursor_left;
        let hr_cursor_right = self.hr_cursor_right;
        let hr_knob_bottom = self.hr_area.1 .1 - 1; // Why did I need this again?

        d.filled_rectangle(self.hr_area.0, self.hr_area.1, &WHITE_PATTERN);
        d.horizontal_line((hr_left, hr_rule_y), hr_right, LINE_BLACK);

        d.vertical_line((hr_cursor_left, hr_top), hr_bottom, LINE_BLACK);
        d.horizontal_line((hr_cursor_left, hr_top), hr_cursor_left + 8, LINE_BLACK);
        d.horizontal_line(
            (hr_cursor_left, hr_knob_bottom),
            hr_cursor_left + 8,
            LINE_BLACK,
        );

        d.vertical_line((hr_cursor_right, hr_top), hr_bottom, LINE_BLACK);
        d.horizontal_line((hr_cursor_right - 8, hr_top), hr_cursor_right, LINE_BLACK);
        d.horizontal_line(
            (hr_cursor_right - 8, hr_knob_bottom),
            hr_cursor_right,
            LINE_BLACK,
        );
    }

    fn draw_v_ruler(&mut self, med: &mut dyn Mediator) {
        let d = med.borrow_mut_desktop();
        let ((vr_left, vr_top), (vr_right, vr_bottom)) = self.vr_area;
        let vr_rule_x = (vr_left + vr_right) >> 1;
        let vr_cursor_top = self.vr_cursor_top;
        let vr_cursor_bottom = self.vr_cursor_bottom;

        d.filled_rectangle(self.vr_area.0, self.vr_area.1, &WHITE_PATTERN);
        d.vertical_line((vr_rule_x, vr_top), vr_bottom, LINE_BLACK);

        d.horizontal_line((vr_left, vr_cursor_top), vr_right, LINE_BLACK);
        d.vertical_line((vr_left, vr_cursor_top), vr_cursor_top + 8, LINE_BLACK);
        d.vertical_line((vr_right - 1, vr_cursor_top), vr_cursor_top + 8, LINE_BLACK);

        d.horizontal_line((vr_left, vr_cursor_bottom), vr_right, LINE_BLACK);
        d.vertical_line(
            (vr_left, vr_cursor_bottom - 8),
            vr_cursor_bottom,
            LINE_BLACK,
        );
        d.vertical_line(
            (vr_right - 1, vr_cursor_bottom - 8),
            vr_cursor_bottom,
            LINE_BLACK,
        );
    }
}

/// Tell the host environment that we are equipped to represent the whole application.
impl<'l, 'f> AppController for ToyBoxApp<'l, 'f> {}

/// Tell the host environment we can determine the application life-cycle.
impl<'l, 'f> AppEventSink for ToyBoxApp<'l, 'f> {
    fn request_quit(&self) -> bool {
        // We have no reason to deny quitting, so yes.
        true
    }
}

/// Sink for host environment mouse events.
impl<'l, 'f> MouseEventSink<()> for ToyBoxApp<'l, 'f> {
    fn pointer_moved(&mut self, med: &mut dyn Mediator, pt: Point) {
        self.mouse_pt = pt;

        // Let the gadgets handle their own pointer motion events.

        match self.xyprop.pointer_moved(med, pt) {
            PropGadgetEvent::KnobMoved(((left, top), (right, bottom))) => {
                self.hr_cursor_left = left;
                self.vr_cursor_top = top;
                self.hr_cursor_right = right - 1;
                self.vr_cursor_bottom = bottom - 1;
            }

            _ => (),
        }

        match self.vprop.pointer_moved(med, pt) {
            PropGadgetEvent::KnobMoved(((_, top), (_, bottom))) => {
                self.vr_cursor_top = top;
                self.vr_cursor_bottom = bottom - 1;
            }

            _ => (),
        }

        match self.hprop.pointer_moved(med, pt) {
            PropGadgetEvent::KnobMoved(((left, _), (right, _))) => {
                self.hr_cursor_left = left;
                self.hr_cursor_right = right - 1;
            }

            _ => (),
        }

        let _ = self.quit_btn.pointer_moved(med, pt);

        // Now let's consider pointer motion events for what the user
        // thinks are custom gadgets.

        match self.selected {
            Selectable::LeftRulerKnob => {
                let new_x = pt.0;

                if (self.hr_area.0 .0 <= new_x) && (new_x < self.hr_cursor_right - 16) {
                    self.hr_cursor_left = new_x;
                }
            }

            Selectable::RightRulerKnob => {
                let new_x = pt.0;

                if (self.hr_cursor_left + 16 <= new_x) && (new_x < self.hr_area.1 .0) {
                    self.hr_cursor_right = new_x;
                }
            }

            Selectable::TopRulerKnob => {
                let new_y = pt.1;

                if (self.vr_area.0 .1 <= new_y) && (new_y < self.vr_cursor_bottom - 16) {
                    self.vr_cursor_top = new_y;
                }
            }

            Selectable::BottomRulerKnob => {
                let new_y = pt.1;

                if (self.vr_cursor_top + 16 <= new_y) && (new_y < self.vr_area.1 .1) {
                    self.vr_cursor_bottom = new_y;
                }
            }

            _ => (),
        }

        self.draw_rulers(med);
        self.draw_prop_gadgets(med);
        med.repaint_all();
    }

    fn button_down(&mut self, med: &mut dyn Mediator) {
        // Handle button events for the regular gadgets.

        let _ = self.xyprop.button_down(med);
        let _ = self.vprop.button_down(med);
        let _ = self.hprop.button_down(med);
        let _ = self.quit_btn.button_down(med);

        // Handle button events for the custom gadgets.

        if self.mouse_in_hr_left_cursor() {
            self.selected = Selectable::LeftRulerKnob;
        } else if self.mouse_in_hr_right_cursor() {
            self.selected = Selectable::RightRulerKnob;
        } else if self.mouse_in_vr_top_cursor() {
            self.selected = Selectable::TopRulerKnob;
        } else if self.mouse_in_vr_bottom_cursor() {
            self.selected = Selectable::BottomRulerKnob;
        }
    }

    fn button_up(&mut self, med: &mut dyn Mediator) {
        // Handle button events for the regular gadgets.

        let _ = self.xyprop.button_up(med);
        let _ = self.vprop.button_up(med);
        let _ = self.hprop.button_up(med);

        match self.quit_btn.button_up(med) {
            PushButtonEvent::Clicked => {
                med.quit();
            }
            _ => (),
        }

        // Handle button events for the custom gadgets.

        self.selected = Selectable::None;
    }

    fn enter(&mut self, med: &mut dyn Mediator, at: Point) {
        let _ = self.xyprop.enter(med, at);
        let _ = self.vprop.enter(med, at);
        let _ = self.hprop.enter(med, at);
        let _ = self.quit_btn.enter(med, at);
    }

    fn leave(&mut self, med: &mut dyn Mediator) {
        let _ = self.xyprop.leave(med);
        let _ = self.vprop.leave(med);
        let _ = self.hprop.leave(med);
        let _ = self.quit_btn.leave(med);
    }
}

impl<'l, 'f> ToyBoxApp<'l, 'f> {
    fn mouse_in_hr_left_cursor(&self) -> bool {
        let cursor_left = self.hr_cursor_left;
        let cursor_top = self.hr_area.0 .1;
        let cursor_right = cursor_left + 8;
        let cursor_bottom = self.hr_area.1 .1;

        let cursor_area = ((cursor_left, cursor_top), (cursor_right, cursor_bottom));
        rect_contains(cursor_area, self.mouse_pt)
    }

    fn mouse_in_hr_right_cursor(&self) -> bool {
        let cursor_right = self.hr_cursor_right;
        let cursor_bottom = self.hr_area.1 .1;
        let cursor_left = cursor_right - 8;
        let cursor_top = self.hr_area.0 .1;

        let cursor_area = ((cursor_left, cursor_top), (cursor_right, cursor_bottom));
        rect_contains(cursor_area, self.mouse_pt)
    }

    fn mouse_in_vr_top_cursor(&self) -> bool {
        let cursor_left = self.vr_area.0 .0;
        let cursor_top = self.vr_cursor_top;
        let cursor_right = cursor_left + 8;
        let cursor_bottom = cursor_top + 8;

        let cursor_area = ((cursor_left, cursor_top), (cursor_right, cursor_bottom));
        rect_contains(cursor_area, self.mouse_pt)
    }

    fn mouse_in_vr_bottom_cursor(&self) -> bool {
        let cursor_left = self.vr_area.0 .0;
        let cursor_top = self.vr_cursor_bottom - 8;
        let cursor_right = cursor_left + 8;
        let cursor_bottom = cursor_top + 8;

        let cursor_area = ((cursor_left, cursor_top), (cursor_right, cursor_bottom));
        rect_contains(cursor_area, self.mouse_pt)
    }
}
