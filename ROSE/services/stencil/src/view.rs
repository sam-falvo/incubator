//! Views

use crate::mediator::Mediator;
use crate::types::{Rect, Point};

/// An interface for things which can be drawn to a stencil.
pub trait View {
    /// Draw the thing to the desktop stencil,
    /// as returned by [[Mediator::borrow_mut_desktop]].
    // ISSUE: Should we just take a &mut Stencil directly here?
    fn draw(&mut self, med: &mut dyn Mediator);
}

/// Answers `true` if and only if the point `p` is contained
/// within the rectangle `r`.
pub fn rect_contains(r: Rect, p: Point) -> bool {
    let ((left, top), (right, bottom)) = r;
    let (x, y) = p;

    (left <= x) && (x < right) && (top <= y) && (y < bottom)
}

