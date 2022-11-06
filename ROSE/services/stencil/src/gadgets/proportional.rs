//! Support for proportional gadgets.
//!
//! Proportional gadgets are used
//! whenever you need
//! analog input from a user.
//! For example,
//! three proportional gadgets
//! can be used
//! to implement a red/green/blue color selector.
//!
//! They can also be used to
//! allow a user to scroll a viewport
//! over a document
//! too large to display all at once.
//!
//! The proportional gadgets
//! supported in this module
//! are largely inspired
//! by those found in AmigaOS' Intuition library.

use crate::types::{Rect, Point};
use crate::utils::{LINE_BLACK, WHITE_PATTERN};
use crate::stencil::{Draw, Pattern};
use crate::events::MouseEventSink;
use crate::mediator::Mediator;
use crate::view::{View, rect_contains};

/// This is the 8x8 pixel pattern used to draw a proportional gadget's track.
pub static PROP_TRACK_PATTERN: Pattern = [
    0b11101110,
    0b11011101,
    0b10111011,
    0b01110111,
    0b11101110,
    0b11011101,
    0b10111011,
    0b01110111,
];

/// Maintains the appearance of a proportional gadget.
pub struct PropGadgetView {
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

impl PropGadgetView {
    /// Creates a new proportional gadget model.
    /// The `track` rectangle defines
    /// the largest rectangle the knob can occupy.
    /// By default, the knob will occupy the entire track.
    /// The borders for the gadget will surround the track rectangle.
    ///
    /// Use [[PropGadgetView::set_knob]] to set the knob size and position within the `track` rectangle.
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
    /// The knob rectangle `k` will be clipped to the area set by the gadget's track.
    pub fn set_knob(&mut self, k: Rect) {
        let ((left, top), (right, bottom)) = k;
        self.knob = (
            (left.max(self.track.0.0), top.max(self.track.0.1)),
            (right.min(self.track.1.0), bottom.min(self.track.1.1))
        );
    }

    /// Retrieves the current knob rectangle.
    pub fn get_knob(&self) -> Rect {
        self.knob
    }

    /// Retrieves the track rectangle.
    pub fn get_track(&self) -> Rect {
        self.track
    }
}

/// Events unique to the proportional gadget.
pub enum PropGadgetEvent {
    /// No event recognized.
    None,

    /// The user has performed an action which requests the knob be moved
    /// to a new location.  Use [[PropGadgetView::set_knob]] to acknowledge
    /// the movement.
    KnobMoved(Rect),
}

impl MouseEventSink<PropGadgetEvent> for PropGadgetView {
    /// Handle mouse motion events.
    ///
    /// If the user is currently moving the knob of the gadget,
    /// [[PropGadgetEvent::KnobMoved]] events will be returned.
    /// **NOTE:** This *will not* update the knob position.
    /// You will still be responsible for calling [[PropGadgetView::set_knob]]
    /// in response to this event.  This gives the application
    /// a chance to filter and/or react to the event before
    /// updating the gadget's knob.
    ///
    /// Otherwise, [[PropGadgetEvent::None]] is returned.
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
            evt = PropGadgetEvent::KnobMoved(new_knob);
        }
        self.mouse_pt = to;
        evt
    }

    /// Handles mouse button-up events.
    ///
    /// Currently, always answers with [[PropGadgetEvent::None]].
    fn button_up(&mut self, _med: &mut dyn Mediator) -> PropGadgetEvent {
        self.grabbed = false;
        PropGadgetEvent::None
    }

    /// Handles mouse button-down events.
    ///
    /// Currently, always answers with [[PropGadgetEvent::None]].
    fn button_down(&mut self, _med: &mut dyn Mediator) -> PropGadgetEvent {
        if self.point_in_knob() {
            self.grabbed = true;
        }
        PropGadgetEvent::None
    }

    /// Handles mouse entry events.
    ///
    /// Currently, always answers with [[PropGadgetEvent::None]].
    fn enter(&mut self, _med: &mut dyn Mediator, _at: Point) -> PropGadgetEvent {
        PropGadgetEvent::None
    }

    /// Handles mouse button-down events.
    ///
    /// Currently, always answers with [[PropGadgetEvent::None]].
    fn leave(&mut self, _med: &mut dyn Mediator) -> PropGadgetEvent {
        PropGadgetEvent::None
    }
}

impl View for PropGadgetView {
    // ISSUE: Should we just pass in a &mut Stencil here?  Or &mut dyn Draw?
    /// Draws the proportional gadget on the desktop stencil associated with the mediator `med`.
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

