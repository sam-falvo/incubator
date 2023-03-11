// vim:ts=4:sw=4:noet:ai

mod lexer {
	pub struct Lexer<'lexer_life> {
		pub chars: core::str::Chars<'lexer_life>,
		pub next: Option<char>,
	}

	impl<'lexer_lifetime> Lexer<'lexer_lifetime> {
		pub fn new_from_str(input: &'lexer_lifetime str) -> Self {
			let mut chars = input.chars();
			let next = chars.next();
			Self {
				chars,
				next,
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
			if let Some(chr) = self.next {
				if chr.is_ascii_digit() {
					let mut number: usize = chr.to_digit(10).unwrap() as usize;

					self.next = self.chars.next();
					loop {
						if let Some(chr) = self.next {
							if chr.is_ascii_digit() {
								number = number * 10 + (chr.to_digit(10).unwrap() as usize);
								self.next = self.chars.next();
							}
							else { break };
						} else { break };
					}
					Some(Token::Number(number))
				}
				else {
					let c = Token::Char(chr);
					self.next = self.chars.next();
					Some(c)
				}
			} else {
				None
			}
		}
	}
}

mod compile {
	use crate::tests::lexer::{Token, Lexer};

	#[derive(Debug,PartialEq)]
	pub enum Ins {
		LoadAImm16(u16),
		Return,
	}

	pub fn compile_from_str(input: &str) -> Vec<Ins> {
		let mut l = Lexer::new_from_str(input);

		let token = l.next();
		if let Some(maybe_number) = token {
			if let Token::Number(n) = maybe_number {
				return vec![
					Ins::LoadAImm16(n as u16),
					Ins::Return,
				];
			}
		}

		// otherwise, syntax error, but we're not there yet.
		vec![]
	}

	#[test]
	fn unsigned_integer() {
		let result = compile_from_str("42");
		assert_eq!(result, vec![
			Ins::LoadAImm16(42),
			Ins::Return,
		]);

		let result = compile_from_str("420");
		assert_eq!(result, vec![
			Ins::LoadAImm16(420),
			Ins::Return,
		]);

		let result = compile_from_str("-42");
		assert_eq!(result, vec![]);
	}
}
