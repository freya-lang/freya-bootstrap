use crate::tokenizer::{Token, TokenData, TokenizationOutput};
use crate::utils::{Span, Spanned};

#[derive(Debug)]
pub(crate) struct Error {
	pub token_index: usize,
	pub source_location: usize,
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
pub(crate) type Name = Spanned<NameData>;
pub(crate) type Statement = Spanned<StatementData>;
pub(crate) type Item = Spanned<ItemData>;
pub(crate) type ParameterOrIndex = Spanned<ParameterOrIndexData>;
pub(crate) type Constructor = Spanned<ConstructorData>;

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
	Value {
		path: Vec<String>,
	},
	Set {
		level: usize,
	},
	Prop,
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
pub(crate) struct NameData {
	pub value: String,
}

#[derive(Debug)]
pub(crate) enum StatementData {
	Let { binding: TypedBinding, body: Expr },
	Return { body: Expr },
}

#[derive(Debug)]
pub(crate) enum ItemData {
	Let {
		binding: TypedBinding,
		body: Expr,
	},
	Type {
		name: Name,
		params_and_indexes: Vec<ParameterOrIndex>,
		universe: Option<Expr>,
		constructors: Vec<Constructor>,
	},
}

#[derive(Debug)]
pub(crate) enum ParameterOrIndexData {
	Parameter { binding: Binding, ascribed_type: Expr },
	Index { ascribed_type: Expr },
}

#[derive(Debug)]
pub(crate) struct ConstructorData {
	name: Name,
	constructor_type: Expr,
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
			source_location: self.peek().span.start,
			token_index: self.index,
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
						TokenData::Asterisk => {},
						TokenData::Question => {},
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
				let initial_span = spool.peek().span;
				let mut final_span = initial_span;

				let mut path = Vec::new();
				path.push(value.clone());

				spool.advance();

				loop {
					let TokenData::DoubleColon = spool.peek().data else {
						break;
					};
					spool.advance();

					let TokenData::Identifier { ref value } = spool.peek().data else {
						return spool.error();
					};
					final_span = spool.peek().span;

					path.push(value.clone());

					spool.advance();
				}

				Ok(Expr::from(ExprData::Value { path }, initial_span, final_span))
			},
			TokenData::Asterisk => {
				let initial_span = spool.peek().span;
				let mut final_span = initial_span;
				spool.advance();

				let mut level = 0;
				loop {
					let TokenData::Apostrophe = spool.peek().data else {
						break;
					};

					final_span = spool.peek().span;
					spool.advance();

					level += 1;
				}

				Ok(Expr::from(ExprData::Set { level }, initial_span, final_span))
			},
			TokenData::Question => {
				let span = spool.peek().span;
				spool.advance();

				Ok(Expr {
					data: ExprData::Prop,
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

impl Name {
	fn parse(spool: &mut Spool) -> Result<Self, Error> {
		let TokenData::Identifier { ref value } = spool.peek().data else {
			return spool.error();
		};
		let value = value.clone();
		let span = spool.peek().span;
		spool.advance();

		Ok(Name {
			data: NameData { value },
			span,
		})
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

				let TokenData::Semicolon = spool.peek().data else {
					return spool.error();
				};
				let final_span = spool.peek().span;
				spool.advance();

				Ok(Item::from(ItemData::Let { binding, body }, initial_span, final_span))
			},
			TokenData::Type => {
				let initial_span = spool.peek().span;
				spool.advance();

				let name = Name::parse(spool)?;

				let mut params_and_indexes = Vec::new();

				if let TokenData::OpenParen = spool.peek().data {
					spool.advance();

					loop {
						if let TokenData::CloseParen = spool.peek().data {
							spool.advance();
							break;
						}

						params_and_indexes.push(ParameterOrIndex::parse(spool)?);

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
				}

				let mut universe = None;

				if let TokenData::Colon = spool.peek().data {
					spool.advance();

					universe = Some(Expr::parse_atomic(spool)?);
				}

				let TokenData::OpenBrace = spool.peek().data else {
					return spool.error();
				};
				spool.advance();

				let mut constructors = Vec::new();
				let final_span;

				loop {
					if let TokenData::CloseBrace = spool.peek().data {
						final_span = spool.peek().span;
						spool.advance();
						break;
					}

					constructors.push(Constructor::parse(spool)?);

					match spool.peek().data {
						TokenData::Comma => {
							spool.advance();
						},
						TokenData::CloseBrace => {
							final_span = spool.peek().span;
							spool.advance();
							break;
						},
						_ => return spool.error(),
					}
				}

				Ok(Item::from(
					ItemData::Type {
						name,
						params_and_indexes,
						universe,
						constructors,
					},
					initial_span,
					final_span,
				))
			},
			_ => spool.error(),
		}
	}
}

impl ParameterOrIndex {
	fn parse(spool: &mut Spool) -> Result<Self, Error> {
		match spool.peek().data {
			TokenData::AtSign => {
				let initial_span = spool.peek().span;
				spool.advance();

				let TokenData::Colon = spool.peek().data else {
					return spool.error();
				};
				spool.advance();

				let ascribed_type = Expr::parse_general(spool)?;

				let final_span = ascribed_type.span;

				Ok(ParameterOrIndex::from(
					ParameterOrIndexData::Index { ascribed_type },
					initial_span,
					final_span,
				))
			},
			_ => {
				let binding = Binding::parse(spool)?;

				let TokenData::Colon = spool.peek().data else {
					return spool.error();
				};
				spool.advance();

				let ascribed_type = Expr::parse_general(spool)?;

				let initial_span = binding.span;
				let final_span = ascribed_type.span;

				Ok(ParameterOrIndex::from(
					ParameterOrIndexData::Parameter { binding, ascribed_type },
					initial_span,
					final_span,
				))
			},
		}
	}
}

impl Constructor {
	fn parse(spool: &mut Spool) -> Result<Self, Error> {
		let name = Name::parse(spool)?;

		let TokenData::Colon = spool.peek().data else {
			return spool.error();
		};
		spool.advance();

		let constructor_type = Expr::parse_general(spool)?;

		let initial_span = name.span;
		let final_span = constructor_type.span;

		Ok(Constructor::from(
			ConstructorData { name, constructor_type },
			initial_span,
			final_span,
		))
	}
}

pub(crate) fn parse(tokenization_output: TokenizationOutput) -> Result<File, Error> {
	let mut spool = Spool::new(tokenization_output);
	File::parse(&mut spool)
}
