use crate::tokenizer::{Token, TokenData, TokenizationOutput};
use crate::utils::{Span, Spanned};

#[derive(Debug)]
pub(crate) struct Error {
	location: usize,
}

struct Spool {
	tokens: Vec<Token>,
	index: usize,
	eof: Token,
}

pub(crate) type File = Spanned<FileData>;
pub(crate) type Expr = Spanned<ExprData>;
pub(crate) type TypedBinding = Spanned<TypedBindingData>;
pub(crate) type Binding = Spanned<BindingData>;
pub(crate) type Statement = Spanned<StatementData>;
pub(crate) type Item = Spanned<ItemData>;

#[derive(Debug)]
pub(crate) struct FileData {
	items: Vec<Item>,
}

#[derive(Debug)]
pub(crate) enum ExprData {
	FnLowercase {
		args: Vec<TypedBinding>,
		return_type: Option<Box<Expr>>,
		body: Box<Expr>,
	},
	FnUppercase {
		args: Vec<TypedBinding>,
		return_type: Box<Expr>,
	},
	Block {
		statements: Vec<Statement>,
	},
	Grouping {
		inner: Box<Expr>,
	},
	Application {
		left: Box<Expr>,
		right: Box<Expr>,
	},
	Binding {
		binding_name: String,
	},
}

#[derive(Debug)]
pub(crate) struct TypedBindingData {
	binding: Binding,
	ascribed_type: Option<Expr>,
}

#[derive(Debug)]
pub(crate) enum BindingData {
	Identifier { binding_name: String },
	Underscore,
}

#[derive(Debug)]
pub(crate) enum StatementData {
	Let { binding: TypedBinding, body: Expr },
	Return { body: Expr },
}

#[derive(Debug)]
pub(crate) enum ItemData {
	Let { binding: TypedBinding, body: Expr },
}

impl Spool {
	fn new(tokenization_output: TokenizationOutput) -> Self {
		Self {
			tokens: tokenization_output.tokens,
			index: 0,
			eof: Token {
				data: TokenData::Eof,
				span: Span {
					start: tokenization_output.end,
					end: tokenization_output.end,
				},
			},
		}
	}

	fn peek(&self) -> &Token {
		self.tokens.get(self.index).unwrap_or(&self.eof)
	}

	fn advance(&mut self) {
		self.index += 1;
	}

	fn is_end(&self) -> bool {
		self.index >= self.tokens.len()
	}

	fn error<T>(&self) -> Result<T, Error> {
		Err(Error {
			location: self.peek().span.start,
		})
	}
}

impl File {
	fn parse(spool: &mut Spool) -> Result<Self, Error> {
		let mut items = Vec::new();

		while !spool.is_end() {
			items.push(Item::parse(spool)?);
		}

		Ok(File {
			data: FileData { items },
			span: Span {
				start: 0,
				end: spool.eof.span.end,
			},
		})
	}
}

impl Expr {
	fn parse_general(spool: &mut Spool) -> Result<Self, Error> {
		match spool.peek().data {
			TokenData::FnLowercase => {
				let initial_span = spool.peek().span;
				spool.advance();

				let TokenData::OpenParen = spool.peek().data else {
					return spool.error();
				};
				spool.advance();

				let mut bindings: Vec<TypedBinding> = Vec::new();

				loop {
					if let TokenData::CloseParen = spool.peek().data {
						spool.advance();
						break;
					}

					bindings.push(TypedBinding::parse(spool)?);

					match spool.peek().data {
						TokenData::Comma => {
							spool.advance();
						},
						TokenData::CloseParen => {
							spool.advance();
							break;
						},
						_ => return spool.error(),
					}
				}

				let mut return_type = None;

				if let TokenData::Arrow = spool.peek().data {
					spool.advance();
					return_type = Some(Box::new(Expr::parse_general(spool)?));

					let TokenData::Comma = spool.peek().data else {
						return spool.error();
					};
					spool.advance();
				}

				let body = Box::new(Expr::parse_general(spool)?);
				let final_span = body.span;

				Ok(Expr::from(
					ExprData::FnLowercase {
						args: bindings,
						return_type,
						body,
					},
					initial_span,
					final_span,
				))
			},
			TokenData::FnUppercase => {
				let initial_span = spool.peek().span;
				spool.advance();

				let TokenData::OpenParen = spool.peek().data else {
					return spool.error();
				};
				spool.advance();

				let mut bindings: Vec<TypedBinding> = Vec::new();

				loop {
					if let TokenData::CloseParen = spool.peek().data {
						spool.advance();
						break;
					}

					bindings.push(TypedBinding::parse(spool)?);

					match spool.peek().data {
						TokenData::Comma => {
							spool.advance();
						},
						TokenData::CloseParen => {
							spool.advance();
							break;
						},
						_ => return spool.error(),
					}
				}

				let TokenData::Arrow = spool.peek().data else {
					return spool.error();
				};
				spool.advance();

				let return_type = Box::new(Expr::parse_general(spool)?);
				let final_span = return_type.span;

				Ok(Expr::from(
					ExprData::FnUppercase {
						args: bindings,
						return_type,
					},
					initial_span,
					final_span,
				))
			},
			_ => {
				let mut exprs = Vec::new();

				loop {
					exprs.push(Expr::parse_atomic(spool)?);

					match spool.peek().data {
						TokenData::OpenBrace => {},
						TokenData::OpenParen => {},
						TokenData::Identifier { .. } => {},
						_ => break,
					}
				}

				let mut iterator = exprs.into_iter();
				let mut out = iterator.next().unwrap();

				for item in iterator {
					let initial_span = out.span;
					let final_span = item.span;

					out = Expr::from(
						ExprData::Application {
							left: Box::new(out),
							right: Box::new(item),
						},
						initial_span,
						final_span,
					);
				}

				Ok(out)
			},
		}
	}

	fn parse_atomic(spool: &mut Spool) -> Result<Self, Error> {
		match spool.peek().data {
			TokenData::OpenBrace => {
				let initial_span = spool.peek().span;
				spool.advance();

				let mut statements = Vec::new();

				loop {
					if let TokenData::CloseBrace = spool.peek().data {
						break;
					}

					let statement = Statement::parse(spool)?;

					statements.push(statement);

					let TokenData::Semicolon = spool.peek().data else {
						return spool.error();
					};
					spool.advance();
				}

				let final_span = spool.peek().span;
				spool.advance();

				Ok(Expr::from(ExprData::Block { statements }, initial_span, final_span))
			},
			TokenData::OpenParen => {
				let initial_span = spool.peek().span;
				spool.advance();

				let expr = Expr::parse_general(spool)?;

				let TokenData::CloseParen = spool.peek().data else {
					return spool.error();
				};
				let final_span = spool.peek().span;
				spool.advance();

				Ok(Expr::from(
					ExprData::Grouping { inner: Box::new(expr) },
					initial_span,
					final_span,
				))
			},
			TokenData::Identifier { ref value } => {
				let span = spool.peek().span;
				let binding_name = value.clone();
				spool.advance();

				Ok(Expr {
					data: ExprData::Binding { binding_name },
					span,
				})
			},
			_ => spool.error(),
		}
	}
}

impl TypedBinding {
	fn parse(spool: &mut Spool) -> Result<Self, Error> {
		let binding = Binding::parse(spool)?;
		let initial_span = binding.span;
		let mut final_span = binding.span;
		let mut ascribed_type = None;

		if let TokenData::Colon = spool.peek().data {
			spool.advance();

			let expr = Expr::parse_general(spool)?;
			final_span = expr.span;
			ascribed_type = Some(expr);
		}

		Ok(TypedBinding::from(
			TypedBindingData { binding, ascribed_type },
			initial_span,
			final_span,
		))
	}
}

impl Binding {
	fn parse(spool: &mut Spool) -> Result<Self, Error> {
		match spool.peek().data {
			TokenData::Underscore => {
				let span = spool.peek().span;
				spool.advance();

				Ok(Binding {
					data: BindingData::Underscore,
					span,
				})
			},
			TokenData::Identifier { ref value } => {
				let span = spool.peek().span;
				let binding_name = value.clone();
				spool.advance();

				Ok(Binding {
					data: BindingData::Identifier { binding_name },
					span,
				})
			},
			_ => return spool.error(),
		}
	}
}

impl Statement {
	fn parse(spool: &mut Spool) -> Result<Self, Error> {
		match spool.peek().data {
			TokenData::Let => {
				let initial_span = spool.peek().span;
				spool.advance();

				let binding = TypedBinding::parse(spool)?;

				let TokenData::Equals = spool.peek().data else {
					return spool.error();
				};
				spool.advance();

				let body = Expr::parse_general(spool)?;
				let final_span = body.span;

				Ok(Statement::from(
					StatementData::Let { binding, body },
					initial_span,
					final_span,
				))
			},
			TokenData::Return => {
				let initial_span = spool.peek().span;
				spool.advance();

				let body = Expr::parse_general(spool)?;
				let final_span = body.span;

				Ok(Statement::from(
					StatementData::Return { body },
					initial_span,
					final_span,
				))
			},
			_ => return spool.error(),
		}
	}
}

impl Item {
	fn parse(spool: &mut Spool) -> Result<Self, Error> {
		let initial_span = spool.peek().span;
		spool.advance();

		let binding = TypedBinding::parse(spool)?;

		let TokenData::Equals = spool.peek().data else {
			return spool.error();
		};
		spool.advance();

		let body = Expr::parse_general(spool)?;

		let TokenData::Semicolon = spool.peek().data else {
			return spool.error();
		};
		let final_span = spool.peek().span;
		spool.advance();

		Ok(Item::from(ItemData::Let { binding, body }, initial_span, final_span))
	}
}

pub(crate) fn parse(tokenization_output: TokenizationOutput) -> Result<File, Error> {
	let mut spool = Spool::new(tokenization_output);
	File::parse(&mut spool)
}
