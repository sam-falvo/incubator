// vim:ts=4:sw=4:et:ai

pub struct Lexer<'input_str_lifetime> {
    pub chars: core::str::Chars<'input_str_lifetime>,
    pub next: Option<char>,
}

impl<'input_lifetime> Lexer<'input_lifetime> {
    pub fn new_from_str(input: &'input_lifetime str) -> Self {
        let mut chars = input.chars();
        let next = chars.next();
        Self { chars, next }
    }

    fn skip(&mut self) {
        self.next = self.chars.next();
    }

    fn lex_octdec_number(&mut self, chr: char, base: u32) -> Option<Token> {
        let mut number: u32 = chr.to_digit(base).unwrap();
        self.skip();
        loop {
            match self.next {
                Some(chr) if chr == '_' => {
                    self.skip();
                }

                Some(chr) if chr.is_ascii_digit() => {
                    number = number * base + chr.to_digit(base).unwrap();
                    self.skip();
                }

                _ => break,
            }
        }
        Some(Token::Number(number as usize))
    }

    fn lex_number(&mut self, chr: char) -> Option<Token> {
        self.lex_octdec_number(chr, 10)
    }

    fn lex_octal_number(&mut self, chr: char) -> Option<Token> {
        self.lex_octdec_number(chr, 8)
    }

    fn lex_hex_number(&mut self) -> Option<Token> {
        self.skip(); // skip the x|X in the 0x prefix.
        let mut number: u32 = 0;

        loop {
            match self.next {
                Some(chr) if chr == '_' => self.skip(),
                Some(chr) if chr.is_ascii_hexdigit() => {
                    number = (number << 4) + chr.to_digit(16).unwrap();
                    self.skip();
                }
                _ => break,
            }
        }
        Some(Token::Number(number as usize))
    }

    fn lex_based_number(&mut self) -> Option<Token> {
        // we already know the current character is '0'.
        self.skip();

        match self.next {
            Some('d') | Some('D') => self.lex_number('0'),
            Some('x') | Some('X') => self.lex_hex_number(),
            Some('q') | Some('Q') | Some('o') | Some('O') => self.lex_octal_number('0'),
            Some(chr) if (chr >= '0') && (chr <= '7') => self.lex_octal_number(chr),
            Some(chr) if chr.is_ascii_digit() => self.lex_number(chr),
            Some(_) => Some(Token::Number(0)),
            None => None,
        }
    }

    fn lex_identifier(&mut self, chr: char) -> Option<Token> {
        // we already know first character starts an identifier.
        let mut id = chr.to_string();
        self.skip();

        loop {
            match self.next {
                Some(chr) if chr.is_lowercase() => id.push(chr),
                Some(chr) if chr.is_uppercase() => id.push(chr),
                Some(chr) if chr.is_ascii_digit() => id.push(chr),
                Some(chr) if chr == '_' => id.push(chr),
                _ => break,
            }
            self.skip();
        }

        Some(Token::Id(id.clone()))
    }
}

#[derive(Debug, PartialEq, Clone, )]
pub enum Token {
    Char(char),
    Number(usize),
    Id(String),
    Let,
}

impl<'input_lifetime> Iterator for Lexer<'input_lifetime> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        // Skip whitespace
        loop {
            match self.next {
                Some(chr) if chr.is_whitespace() => self.skip(),
                _ => break,
            }
        }

        // Found what looks to be the start of another token
        let id = match self.next {
            Some('0') => self.lex_based_number(),
            Some(chr) if chr.is_ascii_digit() => self.lex_number(chr),
            Some(chr) if chr.is_lowercase() => self.lex_identifier(chr),
            Some(chr) if chr.is_uppercase() => self.lex_identifier(chr),
            Some(chr) if chr == '_' => self.lex_identifier(chr),
            Some(chr) => {
                self.skip();
                Some(Token::Char(chr))
            }
            _ => None,
        };

        // filter keywords out from identifiers
        let id = match id {
            Some(Token::Id(ref s)) => {
                match s.as_str() {
                    "let" => Some(Token::Let),
                    &_ => id,
                }
            }

            _ => id,
        };

        id
    }
}
