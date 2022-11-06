//! Support for *proportional* gadgets.
//!
//! Documentation TBD.

use stencil::types::{Rect, Point};
use stencil::utils::{LINE_BLACK, WHITE_PATTERN};
use stencil::stencil::{Draw, Pattern};

use crate::app::{Mediator, View, MouseEventSink};
use crate::app::rect_contains;

static PROP_TRACK_PATTERN: Pattern = [
    0b11101110,
    0b11011101,
    0b10111011,
    0b01110111,
    0b11101110,
    0b11011101,
    0b10111011,
    0b01110111,
];

// ------------------------------------------------------------------------
// Model
// ------------------------------------------------------------------------

/// Maintains the state of a proportional gadget.
pub struct Model {
    /// The rectangle describing the track of the proportional gadget.
    /// **Note:** When rendering the proportional gadget, there will be a
    /// border surrounding the track.  The `track` rectangle *does not*
    /// include the border dimensions.
    track: Rect,

    /// The rectangle describing the knob within the track.
    /// This will always be a sub-rectangle of `track`.
    knob: Rect,

    /// True if the user has grabbed onto the knob; false otherwise.
    grabbed: bool,

    /// Tracks the current mouse pointer position on the screen.
    mouse_pt: Point,
}

impl Model {
    /// Creates a new proportional gadget model.
    /// The `track` rectangle defines
    /// the largest rectangle the knob can occupy.
    /// By default, the knob will occupy the entire track.
    /// The borders for the gadget will surround the track rectangle.
    pub fn new(track: Rect) -> Self {
        Self {
            track,

            knob: track,
            grabbed: false,
            mouse_pt: (0, 0),
        }
    }

    /// Answers true if the mouse pointer lies within the knob.
    fn point_in_knob(&self) -> bool {
        rect_contains(self.knob, self.mouse_pt)
    }

    /// Sets the knob rectangle.
    ///
    /// The knob must be a sub-rectangle of the `track`.
    /// If it is not, weird behavior will ensue.
    pub fn set_knob(&mut self, k: Rect) {
        self.knob = k;
    }
}

pub enum PropGadgetEvent {
    None,
    KnobMoved(Rect),
}

impl MouseEventSink<PropGadgetEvent> for Model {
    fn pointer_moved(&mut self, _med: &mut dyn Mediator, to: Point) -> PropGadgetEvent {
        let mut evt = PropGadgetEvent::None;
        if self.grabbed {
            let dx = to.0 - self.mouse_pt.0;
            let dy = to.1 - self.mouse_pt.1;

            let track_left = self.track.0.0;
            let track_top = self.track.0.1;
            let track_right = self.track.1.0;
            let track_bottom = self.track.1.1;

            let new_left = self.knob.0.0 + dx;
            let new_top = self.knob.0.1 + dy;
            let new_right = self.knob.1.0 + dx;
            let new_bottom = self.knob.1.1 + dy;

            // constraint_left goes positive if there's a correction to be made.
            let constraint_left = (track_left - new_left).max(0);
            let new_left = new_left + constraint_left;
            let new_right = new_right + constraint_left;

            // constraint_right goes negative if there's a correction to be made.
            let constraint_right = (track_right - new_right).min(0);
            let new_left = new_left + constraint_right;
            let new_right = new_right + constraint_right;

            // constraint_top goes positive if there's a correction to be made.
            let constraint_top = (track_top - new_top).max(0);
            let new_top = new_top + constraint_top;
            let new_bottom = new_bottom + constraint_top;

            // constraint_bottom goes negative if there's a correction to be made.
            let constraint_bottom = (track_bottom - new_bottom).min(0);
            let new_top = new_top + constraint_bottom;
            let new_bottom = new_bottom + constraint_bottom;

            let new_knob = ((new_left, new_top), (new_right, new_bottom));
            self.set_knob(new_knob);
            evt = PropGadgetEvent::KnobMoved(new_knob);
        }
        self.mouse_pt = to;
        evt
    }

    fn button_up(&mut self, _med: &mut dyn Mediator) -> PropGadgetEvent {
        self.grabbed = false;
        PropGadgetEvent::None
    }

    fn button_down(&mut self, _med: &mut dyn Mediator) -> PropGadgetEvent {
        if self.point_in_knob() {
            self.grabbed = true;
        }
        PropGadgetEvent::None
    }

    fn enter(&mut self, _med: &mut dyn Mediator, _at: Point) -> PropGadgetEvent {
        PropGadgetEvent::None
    }

    fn leave(&mut self, _med: &mut dyn Mediator) -> PropGadgetEvent {
        PropGadgetEvent::None
    }
}

impl View for Model {
    fn draw(&mut self, med: &mut dyn Mediator) {
        let d = med.borrow_mut_desktop();
        let border = ((self.track.0.0 - 2, self.track.0.1 - 2),
                      (self.track.1.0 + 2, self.track.1.1 + 2));
        d.filled_rectangle(border.0, border.1, &PROP_TRACK_PATTERN);
        d.framed_rectangle(border.0, border.1, LINE_BLACK);
        d.filled_rectangle(self.knob.0, self.knob.1, &WHITE_PATTERN);
        d.framed_rectangle(self.knob.0, self.knob.1, LINE_BLACK);
    }
}

