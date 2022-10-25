//! Useful tools that are frequently used, but which don't really fit anywhere else.

use crate::types::Rect;
use crate::stencil::{Draw, Pattern};

/// The default desktop background pattern (a 50% grey stipple).
pub static DESKTOP_PATTERN: Pattern = [0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55];

/// A pattern consisting of all black pixels.
pub static BLACK_PATTERN: Pattern = [0, 0, 0, 0, 0, 0, 0, 0];

/// A pattern consisting of all white pixels.
pub static WHITE_PATTERN: Pattern = [255, 255, 255, 255, 255, 255, 255, 255];


/// Draws the background for a desktop environment.
///
/// Currently, this function fills the entire stencil with the DESKTOP_PATTERN.
pub fn draw_desktop(st: &mut dyn Draw) {
    st.filled_rectangle((0, 0), st.get_dimensions(), &DESKTOP_PATTERN);
}

/// Draws a simple dialog box onto the provided stencil.
///
/// The `paper` parameter specifies the rectangle of the dialog's "paper" surface.  Any borders to
/// the dialog box will be drawn *around* this rectangle, clipped as appropriate.  This allows a
/// dialog to cover the full stencil surface if required.
pub fn draw_dialog_box(
    st: &mut dyn Draw,
    paper: Rect,
) {
    let ((paper_left, paper_top), (paper_right, paper_bottom)) = paper;

    let border_left = paper_left - 1;
    let border_top = paper_top - 1;
    let border_right = paper_right + 1;
    let border_bottom = paper_bottom + 1;

    let shadow_left = border_left + 1;
    let shadow_top = border_top + 1;
    let shadow_right = border_right + 1;
    let shadow_bottom = border_bottom + 1;

    st.filled_rectangle(
        (shadow_left, shadow_top),
        (shadow_right, shadow_bottom),
        &BLACK_PATTERN,
    );
    st.filled_rectangle(
        (border_left, border_top),
        (border_right, border_bottom),
        &BLACK_PATTERN,
    );
    st.filled_rectangle(
        (paper_left, paper_top),
        (paper_right, paper_bottom),
        &WHITE_PATTERN,
    );
}

