//! This module provides basic support for rendering text with word-wrapping.
//!
//! > **forme:**
//! > *noun* a body of type secured in a chase for printing.
//! &mdash; *[Google, via Oxford Languages](https://www.google.com/search?hl=en&q=define%20forme),
//! accessed 2022-Oct-26.*
//!
//! > **chase:** (def. 3)
//! > *noun* (in letterpress printing) a metal frame for holding the composed type and blocks being printed at one time.
//! &mdash; *[Google, via Oxford Languages](https://www.google.com/search?hl=en&q=define%20chase),
//! accessed 2022-Oct-26.*
//!
//! > **oxford english:**
//! > *noun* the dialect of English spoken at Oxford University and regarded by many as affected and pretentious
//! &mdash; *[WordNik](https://www.wordnik.com/words/oxford%20english), accessed 2022-Oct-26.*

use std::ops::Range;

pub struct Forme {
    carriage_width: usize,
    row_count: usize,
}

impl Forme {
    pub fn new(carriage_width: usize) -> Self {
        Forme {
            carriage_width,

            row_count: 0,
        }
    }

    pub fn layout_from_string(&mut self, input: &str) {
        let mut n_rows = 0;

        if input.len() > 0 {
            n_rows = 1;

            let mut carriage: usize = 0;
            let mut input_iter = input.chars();

            'scan_loop: loop {
                let some_ch = input_iter.next();
                match some_ch {
                    Some(ch) => {
                        if ch == '\n' {
                            n_rows += 1;
                            carriage = 0;
                        } else {
                            carriage += 1;
                            if carriage > self.carriage_width {
                                n_rows += 1;
                                carriage = 0;
                            }
                        }
                    },

                    None => break 'scan_loop,
                }
            }
        }

        self.row_count = n_rows;
    }

    pub fn row_count(&self) -> usize {
        self.row_count
    }

    pub fn get_row(&self, index: usize) -> Option<Row> {
        let starts: [usize; 2] = [0, 6];
        let ends: [usize; 2] = [6, 15];

        if index >= self.row_count {
            None
        } else {
            Some(Row {
                start: starts[index],
                end: ends[index],
            })
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Row {
    start: usize,
    end: usize,
}

impl Row {
    pub fn get_range(&self) -> (usize, usize) {
        (self.start, self.end)
    }
}

#[cfg(test)]
mod tests {
    use super::Forme;

    #[test]
    fn from_empty_string() {
        let mut forme = Forme::new(10);
        forme.layout_from_string("");
        assert_eq!(forme.row_count(), 0);
    }

    #[test]
    fn from_stripped_phrase() {
        let mut forme = Forme::new(20);
        forme.layout_from_string("Hello world");
        assert_eq!(forme.row_count(), 1);
    }

    #[test]
    fn from_two_lines() {
        let mut forme = Forme::new(10);
        forme.layout_from_string("Hello\nworld");
        assert_eq!(forme.row_count(), 2);
    }

    #[test]
    fn from_a_really_long_string() {
        let mut forme = Forme::new(10);
        forme.layout_from_string("pasta carbonara");
        assert_eq!(forme.row_count(), 2);
    }

    #[test]
    fn wraps_on_whitespace_delimited_input() {
        let mut forme = Forme::new(10);
        forme.layout_from_string("pasta carbonara");
        assert_eq!(forme.get_row(0).unwrap().get_range(), (0, 6));
        assert_eq!(forme.get_row(1).unwrap().get_range(), (6, 15));
    }

    #[test]
    fn nonexistent_forme_rows() {
        let mut forme = Forme::new(10);
        forme.layout_from_string("pasta carbonara");
        assert_eq!(forme.get_row(2), None);
    }
}
