use crate::utils::{Span, Spanned};

pub(crate) type Token = Spanned<TokenData>;
type Char = Spanned<char>;

pub(crate) enum TokenData {
	Identifier { value: String },
	FnLowercase,
	FnUppercase,
	Let,
	Return,
	Type,
	Underscore,
	Colon,
	Semicolon,
	Comma,
	OpenParen,
	CloseParen,
	OpenBrace,
	CloseBrace,
	Equals,
	Asterisk,
	Question,
	Apostrophe,
	AtSign,
	DoubleColon,
	Arrow,
	Eof,
}

enum State {
	Neutral,
	Word { buffer: Vec<Char> },
	Symbol { start_span: Span, state: SymbolState },
}

enum SymbolState {
	Hyphen,
	Colon,
}

#[derive(Debug)]
pub(crate) struct Error {
	pub location: usize,
}

struct Spool {
	chars: Vec<Char>,
	index: usize,
}

struct Tokenizer {
	state: State,
	spool: Spool,
	output: Vec<Token>,
}

enum Action {
	Stop,
	Continue,
	Error(Error),
}

fn is_whitespace(chr: char) -> bool {
	matches!(chr, ' ' | '\n' | '\t')
}

fn is_word_start(chr: char) -> bool {
	matches!(chr, 'a' ..= 'z' | 'A' ..= 'Z' | '_')
}

fn is_word_continue(chr: char) -> bool {
	matches!(chr, 'a' ..= 'z' | 'A' ..= 'Z' | '0' ..= '9' | '_')
}

impl Spool {
	fn peek(&self) -> Option<Char> {
		self.chars.get(self.index).copied()
	}

	fn advance(&mut self) {
		self.index += 1;
	}
}

impl Tokenizer {
	fn emit_single_char(&mut self, chr: Char, data: TokenData) -> Action {
		self.output.push(Token { data, span: chr.span });

		self.spool.advance();

		Action::Continue
	}

	fn step(&mut self) -> Action {
		match &mut self.state {
			State::Neutral => {
				let Some(chr) = self.spool.peek() else {
					return Action::Stop;
				};

				if is_whitespace(chr.data) {
					self.spool.advance();

					return Action::Continue;
				}

				if is_word_start(chr.data) {
					self.state = State::Word { buffer: vec![chr] };
					self.spool.advance();

					return Action::Continue;
				}

				match chr.data {
					';' => self.emit_single_char(chr, TokenData::Semicolon),
					',' => self.emit_single_char(chr, TokenData::Comma),
					'(' => self.emit_single_char(chr, TokenData::OpenParen),
					')' => self.emit_single_char(chr, TokenData::CloseParen),
					'{' => self.emit_single_char(chr, TokenData::OpenBrace),
					'}' => self.emit_single_char(chr, TokenData::CloseBrace),
					'=' => self.emit_single_char(chr, TokenData::Equals),
					'*' => self.emit_single_char(chr, TokenData::Asterisk),
					'?' => self.emit_single_char(chr, TokenData::Question),
					'@' => self.emit_single_char(chr, TokenData::AtSign),
					'\'' => self.emit_single_char(chr, TokenData::Apostrophe),
					'-' => {
						self.state = State::Symbol {
							start_span: chr.span,
							state: SymbolState::Hyphen,
						};
						self.spool.advance();

						Action::Continue
					},
					':' => {
						self.state = State::Symbol {
							start_span: chr.span,
							state: SymbolState::Colon,
						};
						self.spool.advance();

						Action::Continue
					},
					_ => Action::Error(Error {
						location: self.spool.index,
					}),
				}
			},
			State::Word { buffer } => {
				let Some(chr) = self.spool.peek() else {
					self.output.push(emit_word_token(&buffer));

					return Action::Stop;
				};

				if !is_word_continue(chr.data) {
					self.output.push(emit_word_token(&buffer));
					self.state = State::Neutral;

					return Action::Continue;
				}

				buffer.push(chr);
				self.spool.advance();

				Action::Continue
			},
			State::Symbol { start_span, state } => match state {
				SymbolState::Hyphen => {
					let Some(Char { data: '>', span }) = self.spool.peek() else {
						return Action::Error(Error {
							location: self.spool.index,
						});
					};

					self.output.push(Token {
						data: TokenData::Arrow,
						span: Span {
							start: start_span.start,
							end: span.end,
						},
					});
					self.state = State::Neutral;
					self.spool.advance();

					Action::Continue
				},
				SymbolState::Colon => {
					let Some(Char { data: ':', span }) = self.spool.peek() else {
						self.output.push(Token {
							data: TokenData::Colon,
							span: *start_span,
						});
						self.state = State::Neutral;

						return Action::Continue;
					};

					self.output.push(Token {
						data: TokenData::DoubleColon,
						span: Span {
							start: start_span.start,
							end: span.end,
						},
					});
					self.state = State::Neutral;
					self.spool.advance();

					Action::Continue
				},
			},
		}
	}
}

fn emit_word_token(buffer: &[Char]) -> Token {
	let value: String = buffer.iter().map(|i| i.data).collect();

	let span = Span {
		start: buffer[0].span.start,
		end: buffer.last().unwrap().span.end,
	};

	let data = match &*value {
		"fn" => TokenData::FnLowercase,
		"Fn" => TokenData::FnUppercase,
		"let" => TokenData::Let,
		"return" => TokenData::Return,
		"type" => TokenData::Type,
		"_" => TokenData::Underscore,
		_ => TokenData::Identifier { value },
	};

	Token { data, span }
}

pub(crate) struct TokenizationOutput {
	pub tokens: Vec<Token>,
	pub end: usize,
}

pub(crate) fn tokenize(text: &str) -> Result<TokenizationOutput, Error> {
	// TODO: make this single pass?

	let mut chars = Vec::new();

	let mut iterator = text.char_indices();
	while let Some((start, data)) = iterator.next() {
		chars.push(Char {
			data,
			span: Span {
				start,
				end: iterator.offset(),
			},
		});
	}

	let mut tokenizer = Tokenizer {
		state: State::Neutral,
		spool: Spool { chars, index: 0 },
		output: Vec::new(),
	};

	loop {
		match tokenizer.step() {
			Action::Continue => {},
			Action::Stop => {
				return Ok(TokenizationOutput {
					tokens: tokenizer.output,
					end: tokenizer.spool.index,
				});
			},
			Action::Error(error) => return Err(error),
		}
	}
}
