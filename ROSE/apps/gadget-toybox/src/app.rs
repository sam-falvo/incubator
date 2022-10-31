use stencil::utils::{draw_desktop, draw_dialog_box};
use stencil::utils::{LINE_BLACK, WHITE_PATTERN};
use stencil::stencil::Stencil;
use stencil::stencil::Draw;
use stencil::types::{Dimension, Point, Rect, Unit};
use stencil::sysfont_bsw_9::SYSTEM_BITMAP_FONT;
use stencil::simple_bitmap_font::SimpleBitmapFont;
use stencil::simple_printer::SimplePrinter;

pub trait AppController: MouseEventSink + AppEventSink {}

pub trait AppEventSink {
    fn request_quit(&self) -> bool;
}

pub trait MouseEventSink {
    fn pointer_moved(&mut self, med: &mut dyn Mediator, to: Point);
    fn button_up(&mut self, med: &mut dyn Mediator);
    fn button_down(&mut self, med: &mut dyn Mediator);
    fn enter(&mut self, med: &mut dyn Mediator, at: Point);
    fn leave(&mut self, med: &mut dyn Mediator);
}

pub trait Mediator {
    fn repaint_all(&mut self);
    fn quit(&mut self);
    fn borrow_mut_desktop(&mut self) -> &mut Stencil;
}

// ------------------------------------------------------------------------

pub fn init_root(med: &mut dyn Mediator) -> Box<dyn AppController> {
    let mut toybox = Box::new(ToyBoxApp::new());
    toybox.draw(med);
    Box::new(ToyBoxApp::new())
}

pub struct ToyBoxApp {
    dbox_area: Rect,
    quit_area: Rect,
    mouse_pt: Point,
    selected: Selectable,
    hr_area: Rect,
    hr_cursor_left: Unit,
    hr_cursor_right: Unit,
}

enum Selectable {
    None,
    QuitButton,
    LeftRulerKnob,
}

impl ToyBoxApp {
    pub fn new() -> Self {
        Self {
            dbox_area: ((8, 8),(240, 56)),
            quit_area: ((248, 8), (312, 28)),
            mouse_pt: (0, 0),
            selected: Selectable::None,
            hr_area: ((24, 16), (224, 24)),
            hr_cursor_left: 24,
            hr_cursor_right: 223,
        }
    }

    fn draw(&mut self, med: &mut dyn Mediator) {
        draw_desktop(med.borrow_mut_desktop());

        // Draw the quit button
        draw_button(med, self.quit_area, "Quit");

        // Draw the window in which our prop gadgets will sit.
        draw_dialog_box(med.borrow_mut_desktop(), self.dbox_area);
        self.draw_rulers(med);
    }

    fn draw_rulers(&mut self, med: &mut dyn Mediator) {
        let d = med.borrow_mut_desktop();
        let ((hr_left, hr_top), (hr_right, hr_bottom)) = self.hr_area;
        let hr_rule_y = (hr_top + hr_bottom) >> 1;
        let hr_cursor_left = self.hr_cursor_left;
        let hr_cursor_right = self.hr_cursor_right;
        let hr_knob_bottom = self.hr_area.1.1 - 1;

        d.filled_rectangle(self.hr_area.0, self.hr_area.1, &WHITE_PATTERN);
        d.horizontal_line((hr_left, hr_rule_y), hr_right, LINE_BLACK);

        d.vertical_line((hr_cursor_left, hr_top), hr_bottom, LINE_BLACK);
        d.horizontal_line((hr_cursor_left, hr_top), hr_cursor_left + 8, LINE_BLACK);
        d.horizontal_line((hr_cursor_left, hr_knob_bottom), hr_cursor_left + 8, LINE_BLACK);

        d.vertical_line((hr_cursor_right, hr_top), hr_bottom, LINE_BLACK);
        d.horizontal_line((hr_cursor_right - 8, hr_top), hr_cursor_right, LINE_BLACK);
        d.horizontal_line((hr_cursor_right - 8, hr_knob_bottom), hr_cursor_right, LINE_BLACK);
    }
}

// TODO: Promote this to standard API somewhere.
fn draw_button(med: &mut dyn Mediator, area: Rect, label: &str) {
    let d = med.borrow_mut_desktop();
    let font = &SYSTEM_BITMAP_FONT;

    let ((btn_left, btn_top), (btn_right, btn_bottom)) = area;
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

    let label_width = text_width(label, &font);
    let btn_width = subview_right - subview_left;
    let label_left = ((btn_width - label_width) >> 1) + btn_left;
    let label_top = subview_top + font.baseline;
    let label_region = ((label_left, label_top), (subview_right, subview_bottom));

    d.filled_rectangle((subview_left, subview_top), (subview_right, subview_bottom), &WHITE_PATTERN);
    d.framed_rectangle((border_left, border_top), (border_right, border_bottom), LINE_BLACK);
    d.horizontal_line((b_shadow_left, b_shadow_top), b_shadow_right, LINE_BLACK);
    d.vertical_line((r_shadow_left, r_shadow_top), r_shadow_bottom, LINE_BLACK);

    let mut p = SimplePrinter::new(d, label_region, &font);
    p.print(label);

    med.repaint_all();
}

// TODO: Promote this to standard API somewhere.
fn text_width(text: &str, font: &SimpleBitmapFont) -> Dimension {
    text.bytes().map(|b| {
        // If not representable in the glyph set of the font, assume the undefined character glyph,
        // which by definition, is always at highest_char+1 mod 256.
        let highest_character = font.highest_char;
        let lowest_character = font.lowest_char;
        let mut glyph_index = b as usize;

        if (b < lowest_character) || (b > highest_character) {
            glyph_index = (highest_character as usize).overflowing_add(1).0;
        }
        glyph_index -= lowest_character as usize;

        // Let's expand this to a valid array index.

        let left_edge = font.left_edges[glyph_index];
        let right_edge = font.left_edges[glyph_index + 1];
        let glyph_width = right_edge - left_edge;

        glyph_width as Dimension
    }).sum()
}

impl AppController for ToyBoxApp { }

impl AppEventSink for ToyBoxApp {
    fn request_quit(&self) -> bool {
        // We have no reason to deny quitting, so yes.
        true
    }
}

impl MouseEventSink for ToyBoxApp {
    fn pointer_moved(&mut self, med: &mut dyn Mediator, pt: Point) {
        self.mouse_pt = pt;

        match self.selected {
            Selectable::LeftRulerKnob => {
                let new_x = pt.0;

                if (self.hr_area.0.0 <= new_x) && (new_x <= self.hr_cursor_right - 16) {
                    self.hr_cursor_left = new_x;
                    self.draw_rulers(med);
                    med.repaint_all();
                }
            }

            _ => (),
        }
    }

    fn button_down(&mut self, med: &mut dyn Mediator) {
        if rect_contains(self.quit_area, self.mouse_pt) {
            self.selected = Selectable::QuitButton;
            med.borrow_mut_desktop().invert_rectangle(self.quit_area.0, self.quit_area.1);
            med.repaint_all();
        } else if self.mouse_in_hr_left_corsor() {
            self.selected = Selectable::LeftRulerKnob;
        }
    }

    fn button_up(&mut self, med: &mut dyn Mediator) {
        match self.selected {
            Selectable::QuitButton => {
                med.borrow_mut_desktop().invert_rectangle(self.quit_area.0, self.quit_area.1);
                med.repaint_all();
                if rect_contains(self.quit_area, self.mouse_pt) {
                    med.quit();
                }
            }

            _ => (),
        }
        self.selected = Selectable::None;
    }

    fn enter(&mut self, _: &mut dyn Mediator, _: Point) {
    }

    fn leave(&mut self, _: &mut dyn Mediator) {
    }
}

fn rect_contains(r: Rect, p: Point) -> bool {
    let ((left, top), (right, bottom)) = r;
    let (x, y) = p;

    (left <= x) && (x < right) && (top <= y) && (y < bottom)
}

impl ToyBoxApp {
    fn mouse_in_hr_left_corsor(&self) -> bool {
        let cursor_left = self.hr_cursor_left;
        let cursor_top = self.hr_area.0.1;
        let cursor_right = cursor_left + 8;
        let cursor_bottom = self.hr_area.1.1;

        let cursor_area = ((cursor_left, cursor_top), (cursor_right, cursor_bottom));
        rect_contains(cursor_area, self.mouse_pt)
    }

}
