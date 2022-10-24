//! Useful utilities that are frequently used.
//!
//! This module contains functionality that is hard to categorize more precisely, but which remains
//! a useful toolbox for application developers.

use crate::types::Unit;
use crate::stencil::{Draw, Pattern};

/// The default desktop background pattern (a 50% grey stipple).
pub static DESKTOP_PATTERN: Pattern = [0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55];

/// A pattern consisting of all black pixels.
pub static BLACK_PATTERN: Pattern = [0, 0, 0, 0, 0, 0, 0, 0];

/// A pattern consisting of all white pixels.
pub static WHITE_PATTERN: Pattern = [255, 255, 255, 255, 255, 255, 255, 255];


/// Draws the background for a desktop environment.
pub fn draw_desktop(st: &mut impl Draw) {
    st.filled_rectangle((0, 0), st.get_dimensions(), &DESKTOP_PATTERN);
}

/// Draws a simple dialog box onto the provided stencil.
pub fn draw_dialog_box(
    st: &mut impl Draw,
    paper_left: Unit,
    paper_top: Unit,
    paper_right: Unit,
    paper_bottom: Unit,
) {
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

