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

/// Trait implemented by fonts.
// should this be moved into a centralized font module?
pub trait FontProvider {
    /// Returns the width of the rendered glyph
    /// for the requested character `char`.
    fn get_glyph_width(&self, ch: char) -> usize;
}

/// A line is described as a span of text in a (possibly, gap) buffer
/// which starts at character `start` and ends at character `end`.
/// Like other measurements in ROSE, `start` is inclusive while `end` is exclusive.
pub type Line = (usize, usize);

/// This structure implements the state for the word-wrapping algorithm.
pub struct Forme<'a> {
    /// A collection of [[Line]]s mapping back to the source document.
    lines: Vec<Line>,

    /// The width of the virtual carriage.
    margin: usize,

    /// Tracks the virtual print head location relative to the virtual carriage.
    head: usize,

    /// Tracks the start of the current line in the source text.
    start: usize,

    /// Tracks the end of the current line in the source text.
    end: usize,

    /// Tracks the location of the most recently known breaking whitespace.
    brk: Option<usize>,

    /// True if we've reached a terminal state.
    done: bool,

    /// True if we just processed a newline request.
    newline: bool,

    /// The font provider which measures glyph widths for us.
    font_provider: &'a dyn FontProvider,
}

impl<'a> Forme<'a> {
    /// Create a new Forme.
    ///
    /// The `margin` parameter specifies the virtual carriage width.
    pub fn new(margin: usize, font_provider: &'a dyn FontProvider) -> Self {
        Self {
            margin,
            font_provider,

            lines: Vec::new(),
            head: 0,
            start: 0,
            end: 0,
            brk: None,
            done: true, // not a typo
            newline: false,
        }
    }

    /// Resets the forme back to its default state.
    pub fn clear(&mut self) {
        self.lines.clear();
        self.head = 0;
        self.start = 0;
        self.end = 0;
        self.brk = None;
        self.done = false;
        self.newline = false;
    }

    /// Processes a string of text to determine
    /// how many lines it would take up
    /// if printed with the selected font
    /// on a surface with the configured width.
    pub fn layout_from_str(&mut self, s: &str) {
        self.clear();
        for ch in s.chars() {
            match ch {
                _ if is_line_break(ch) => self.handle_space(ch),
                _ if is_paragraph_break(ch) => self.paragraph_break(),
                _ => self.handle_char(ch),
            }

            if self.head > self.margin {
                self.breach_margin();
            }
        }
        self.terminate_input();
    }

    /// Handles a non-breaking whitespace or graphic character.
    ///
    /// # Safety
    ///
    /// This function does not perform wrapping.
    fn handle_char(&mut self, ch: char) {
        if !self.done {
            self.head += self.font_provider.get_glyph_width(ch);
            self.end += 1;
            self.newline = false;
        }
    }

    /// Handles a breaking whitespace character.
    ///
    /// # Safety
    ///
    /// This function does not perform wrapping.
    fn handle_space(&mut self, ch: char) {
        if !self.done {
            let end = self.end;
            self.brk = Some(end);
            self.head += self.font_provider.get_glyph_width(ch);
            self.end = end + 1;
        }
    }

    /// Handles a new line/paragraph break.
    fn paragraph_break(&mut self) {
        if !self.done {
            let end = self.end + 1;

            self.lines.push((self.start, end));
            self.start = end;
            self.newline = true;
            self.brk = None;
            self.head = 0;
        }
    }

    /// Process a margin breach by either line- or word-wrapping as appropriate.
    fn breach_margin(&mut self) {
        if !self.done {
            match self.brk {
                None => {
                    let end = self.end;
                    self.lines.push((self.start, end - 1));
                    self.start = end - 1;
                    self.head = 0;
                },

                Some(brk) => {
                    self.lines.push((self.start, brk + 1));
                    self.start = brk + 1;
                    self.head = 0;
                }
            }
        }
    }

    /// Process the end of input condition.
    fn terminate_input(&mut self) {
        if !self.done {
            if self.newline {
                // Nothing to be done; the last line has already been pushed
                // onto the lines vector.
            } else {
                if self.end > self.start {
                    self.lines.push((self.start, self.end));
                }
            }
            self.done = true;
        }
    }
    
    /// Borrow the lines vector.
    pub fn borrow_lines(&self) -> &Vec<Line> {
        &self.lines
    }
}

/// Answers true if the character is a good point to break a line.
fn is_line_break(ch: char) -> bool {
    // Taken from https://en.wikipedia.org/wiki/Whitespace_character
    // Accessed: 2022-Oct-27
    //
    // This is totally overkill if you're just layout out plain ASCII documents.

    // Ignore line breaks; those are covered in is_paragraph_break.
    match ch {
        '\u{0009}' => true,
        '\u{0020}' => true,
        '\u{1680}' => true,
        '\u{2000}' => true,
        '\u{2001}' => true,
        '\u{2002}' => true,
        '\u{2003}' => true,
        '\u{2004}' => true,
        '\u{2005}' => true,
        '\u{2006}' => true,
        '\u{2007}' => true,
        '\u{2009}' => true,
        '\u{200A}' => true,
        '\u{205F}' => true,
        '\u{3000}' => true,

        // Not technically whitespace, but Wikipedia says these will function like one for our
        // purposes.
        '\u{180E}' => true,
        '\u{200B}' => true,
        '\u{200C}' => true,
        '\u{200D}' => true,

        _ => false,
    }
}

/// Answers true if the character is a paragraph break (an explicit line break).
fn is_paragraph_break(ch: char) -> bool {
    // Taken from https://en.wikipedia.org/wiki/Whitespace_character
    // Accessed: 2022-Oct-27
    //
    // This is totally overkill if you're just layout out plain ASCII documents.

    match ch {
        '\u{000A}' => true,
        '\u{000B}' => true,
        '\u{000C}' => true,
        '\u{000D}' => true,
        '\u{0085}' => true,
        '\u{2028}' => true,
        '\u{2029}' => true,

        _ => false,
    }
}


#[cfg(test)]
mod tests {
    use super::FontProvider;
    use super::Forme;

    struct MyFont {}

    impl FontProvider for MyFont {
        fn get_glyph_width(&self, _ch: char) -> usize {
            1
        }
    }

    #[test]
    fn empty_str() {
        let fp = MyFont{};
        let mut f = Forme::new(10, &fp);
        f.layout_from_str("");
        assert_eq!(f.borrow_lines().len(), 0);
    }

    #[test]
    fn pasta() {
        let fp = MyFont{};
        let mut f = Forme::new(10, &fp);
        f.layout_from_str("pasta");
        assert_eq!(*f.borrow_lines().get(0).unwrap(), (0, 5));

        f.layout_from_str("pasta ");
        assert_eq!(*f.borrow_lines().get(0).unwrap(), (0, 6));
    }

    #[test]
    fn pasta_carbonara() {
        let fp = MyFont{};
        let mut f = Forme::new(10, &fp);
        f.layout_from_str("pasta carbonara");
        assert_eq!(f.borrow_lines().len(), 2);
        assert_eq!(*f.borrow_lines().get(0).unwrap(), (0, 6));
        assert_eq!(*f.borrow_lines().get(1).unwrap(), (6, 15));
    }

    #[test]
    fn pasta_is_delicious() {
        let fp = MyFont{};
        let mut f = Forme::new(10, &fp);
        f.layout_from_str("pasta is delicious");
        assert_eq!(f.borrow_lines().len(), 2);
        assert_eq!(*f.borrow_lines().get(0).unwrap(), (0, 9));
        assert_eq!(*f.borrow_lines().get(1).unwrap(), (9, 18));
    }
}
