#[derive(Clone, Copy, Debug)]
pub(crate) struct Span {
	pub start: usize,
	pub end: usize,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Spanned<T> {
	pub data: T,
	pub span: Span,
}

impl<T> Spanned<T> {
	pub(crate) fn from(data: T, initial_span: Span, final_span: Span) -> Self {
		Self {
			data,
			span: Span {
				start: initial_span.start,
				end: final_span.end,
			},
		}
	}
}
