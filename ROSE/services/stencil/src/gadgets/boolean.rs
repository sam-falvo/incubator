//! Support for boolean gadgets.

use crate::mediator::Mediator;
use crate::types::{Point, Rect};
use crate::view::{View, rect_contains};
use crate::events::MouseEventSink;
use crate::simple_printer::SimplePrinter;
use crate::utils::{LINE_BLACK, WHITE_PATTERN};
use crate::simple_bitmap_font::{SimpleBitmapFont, text_width};
use crate::sysfont_bsw_9::SYSTEM_BITMAP_FONT;
use crate::stencil::Draw;

/// Maintains the appearance of a boolean gadget.
///
/// The lifetime `'l` corresponds to the lifetime of the label.
/// The lifetime `'f` corresponds to the lifetime of the font.
/// Both must obviously live at least as long as the PushButtonView itself.
pub struct PushButtonView<'l, 'f> {
    /// The rectangle occupied by the interior of the button.
    area: Rect,

    /// True if the user clicked inside the gadget;
    /// false when the user releases the mouse button.
    selected: bool,

    /// Tracks the current mouse position.
    mouse_pt: Point,

    // TODO:
    // The following fields really belongs in a separate sub-view
    // structure of some kind.  Refactor accordingly.

    /// Label to print in the button.
    label: &'l str,

    /// Font to print the label in.
    font: &'f SimpleBitmapFont<'f>,
}

impl<'l, 'f> PushButtonView<'l, 'f> {
    /// Creates a new boolean gadget view.
    ///
    /// The `label` must fit inside the gadget's `area` rectangle
    /// when printed in the given `font`.
    ///
    /// By default, the gadget is unselected.
    pub fn new(area: Rect, label: &'l str, font: &'f SimpleBitmapFont) -> Self {
        Self {
            area,
            label,
            font,

            selected: false,
            mouse_pt: (0, 0),
        }
    }

    /// Inverts the gadget's appearance.
    pub fn invert(&mut self, med: &mut dyn Mediator) {
        med.borrow_mut_desktop().invert_rectangle(self.area.0, self.area.1);
    }

    fn is_hot(&self) -> bool {
        rect_contains(self.area, self.mouse_pt)
    }
}

impl<'l, 'f> View for PushButtonView<'l, 'f> {
    fn draw(&mut self, med: &mut dyn Mediator) {
        let d = med.borrow_mut_desktop();
        let font = &SYSTEM_BITMAP_FONT;

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

        let label_width = text_width(self.label, self.font);
        let btn_width = subview_right - subview_left;
        let label_left = ((btn_width - label_width) >> 1) + btn_left;
        let label_top = subview_top + font.baseline;
        let label_region = ((label_left, label_top), (subview_right, subview_bottom));

        d.filled_rectangle(
            (subview_left, subview_top),
            (subview_right, subview_bottom),
            &WHITE_PATTERN,
        );
        d.framed_rectangle(
            (border_left, border_top),
            (border_right, border_bottom),
            LINE_BLACK,
        );
        d.horizontal_line((b_shadow_left, b_shadow_top), b_shadow_right, LINE_BLACK);
        d.vertical_line((r_shadow_left, r_shadow_top), r_shadow_bottom, LINE_BLACK);

        let mut p = SimplePrinter::new(d, label_region, self.font);
        p.print(self.label);
    }
}

pub enum PushButtonEvent {
    None,
    Clicked,
}

impl<'l, 'f> MouseEventSink<PushButtonEvent> for PushButtonView<'l, 'f> {
    fn pointer_moved(&mut self, med: &mut dyn Mediator, to: Point) -> PushButtonEvent {
        self.mouse_pt = to;
        if !self.is_hot() && self.selected {
            self.invert(med);
            med.repaint_all();
            self.selected = false;
        }
        PushButtonEvent::None
    }

    fn button_up(&mut self, _: &mut dyn Mediator) -> PushButtonEvent {
        let evt = if self.selected && self.is_hot() {
            PushButtonEvent::Clicked
        } else {
            PushButtonEvent::None
        };
        self.selected = false;
        evt
    }

    fn button_down(&mut self, med: &mut dyn Mediator) -> PushButtonEvent {
        if rect_contains(self.area, self.mouse_pt) {
            self.selected = true;
            self.invert(med);
            med.repaint_all();
        }
        PushButtonEvent::None
    }

    fn enter(&mut self, med: &mut dyn Mediator, at: Point) -> PushButtonEvent {
        self.pointer_moved(med, at)
    }

    fn leave(&mut self, _: &mut dyn Mediator) -> PushButtonEvent {
        PushButtonEvent::None
    }
}

