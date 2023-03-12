// vim:ts=4:sw=4:et:ai

mod lexer {
    pub struct Lexer<'lexer_life> {
        pub chars: core::str::Chars<'lexer_life>,
        pub next: Option<char>,
    }

    impl<'lexer_lifetime> Lexer<'lexer_lifetime> {
        pub fn new_from_str(input: &'lexer_lifetime str) -> Self {
            let mut chars = input.chars();
            let next = chars.next();
            Self { chars, next }
        }

        fn skip(&mut self) {
            self.next = self.chars.next();
        }

        fn lex_number(&mut self, chr: char) -> Option<Token> {
            let mut number: usize = chr.to_digit(10).unwrap() as usize;

            self.skip();
            loop {
                if let Some(chr) = self.next {
                    match chr {
                        '_' => {
                            self.skip();
                        }

                        _ if chr.is_ascii_digit() => {
                            number = number * 10 + (chr.to_digit(10).unwrap() as usize);
                            self.skip();
                        }

                        _ => break,
                    }
                } else {
                    break;
                };
            }
            Some(Token::Number(number))
        }

        fn lex_octal_number(&mut self, chr: char) -> Option<Token> {
            let mut number: usize = chr.to_digit(8).unwrap() as usize;

            self.skip();
            loop {
                if let Some(chr) = self.next {
                    match chr {
                        '_' => {
                            self.skip();
                        }

                        _ if chr.is_ascii_digit() => {
                            number = (number << 3) + (chr.to_digit(8).unwrap() as usize);
                            self.skip();
                        }

                        _ => break,
                    }
                } else {
                    break;
                };
            }
            Some(Token::Number(number))
        }

        fn lex_hex_number(&mut self) -> Option<Token> {
            self.skip();    // skip the x|X in the 0x prefix.
            let mut number: usize = 0;

            loop {
                if let Some(chr) = self.next {
                    match chr {
                        '_' => {
                            self.skip();
                        }

                        _ if chr.is_ascii_hexdigit() => {
                            number = (number << 4) + (chr.to_digit(16).unwrap() as usize);
                            self.skip();
                        }

                        _ => break,
                    }
                } else {
                    break;
                };
            }
            Some(Token::Number(number))
        }

        fn lex_based_number(&mut self) -> Option<Token> {
            // we already know the current character is '0'.
            self.skip();

            if let Some(chr) = self.next {
                match chr {
                    'd' | 'D' => self.lex_number('0'),
                    'x' | 'X' => self.lex_hex_number(),
                    'q' | 'Q' | 'o' | 'O' => self.lex_octal_number('0'),
                    _ if (chr >= '0') && (chr <= '7') => self.lex_octal_number(chr),
                    _ => self.lex_number(chr),
                }
            } else {
                None
            }
        }
    }

    #[derive(Debug, PartialEq)]
    pub enum Token {
        Char(char),
        Number(usize),
    }

    impl<'lexer_lifetime> Iterator for Lexer<'lexer_lifetime> {
        type Item = Token;

        fn next(&mut self) -> Option<Token> {
            match self.next {
                Some(chr) => {
                    if chr == '0' {
                        self.lex_based_number()
                    } else if chr.is_ascii_digit() {
                        self.lex_number(chr)
                    } else {
                        let c = Token::Char(chr);
                        self.skip();
                        Some(c)
                    }
                }

                _ => None,
            }
        }
    }
}

mod compile {
    use crate::tests::lexer::{Lexer, Token};

    #[derive(Debug, PartialEq)]
    pub enum Ins {
        LoadAImm16(u16),
        Return,
    }

    pub fn compile_from_str(input: &str) -> Vec<Ins> {
        let mut l = Lexer::new_from_str(input);

        let next = l.next();
        if let Some(tok) = next {
            match tok {
                Token::Char(ch) if ch == '-' => {
                    let next = l.next();
                    if let Some(maybe_number) = next {
                        if let Token::Number(n) = maybe_number {
                            vec![Ins::LoadAImm16((!n).wrapping_add(1) as u16), Ins::Return]
                        } else {
                            vec![]
                        }
                    } else {
                        vec![]
                    }
                },

                Token::Number(n) => {
                    vec![Ins::LoadAImm16(n as u16), Ins::Return]
                },

                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    #[test]
    fn unsigned_integer() {
        let result = compile_from_str("42");
        assert_eq!(result, vec![Ins::LoadAImm16(42), Ins::Return,]);

        let result = compile_from_str("420");
        assert_eq!(result, vec![Ins::LoadAImm16(420), Ins::Return,]);

        let result = compile_from_str("49_152");
        assert_eq!(result, vec![Ins::LoadAImm16(49152), Ins::Return,]);

        let result = compile_from_str("0xC000");
        assert_eq!(result, vec![Ins::LoadAImm16(49152), Ins::Return,]);

        let result = compile_from_str("0q10");
        assert_eq!(result, vec![Ins::LoadAImm16(8), Ins::Return,]);

        // I really hate C-style octal syntax.  AT&T should be ashamed of
        // themselves and should feel bad.
        let result = compile_from_str("010");
        assert_eq!(result, vec![Ins::LoadAImm16(8), Ins::Return,]);

        let result = compile_from_str("080");
        assert_eq!(result, vec![Ins::LoadAImm16(80), Ins::Return,]);

        let result = compile_from_str("-42");
        assert_eq!(result, vec![Ins::LoadAImm16(0xFFD6), Ins::Return,]);
    }
}
