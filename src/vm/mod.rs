mod mapping;
mod output;
mod term;

use std::sync::atomic::{AtomicUsize, Ordering};

use self::mapping::Mapping;
use self::term::{Term, TermLink};

#[derive(Clone, Copy)]
struct Nominal(usize);

impl Nominal {
	fn new() -> Self {
		static COUNTER: AtomicUsize = AtomicUsize::new(0);

		Self(COUNTER.fetch_add(1, Ordering::Relaxed))
	}
}

fn settle(link: TermLink) {
	match link.extract() {
		Term::Lambda { .. } => return,
		Term::Binding { .. } => return,
		Term::Application {
			mapping: application_mapping,
			function,
			argument: application_argument,
		} => match function.extract() {
			Term::Lambda {
				mapping: lambda_mapping,
				body,
				argument: lambda_argument,
			} => {
				if let Some(lambda_argument) = lambda_argument {
					let lambda_argument = &mut *lambda_argument.modify();
					let mut application_argument = application_argument.extract();

					let Term::Binding { mapping: outer_mapping } = lambda_argument else {
						unreachable!();
					};
					if let Some(inner_mapping) = application_argument.mapping_mut() {
						*inner_mapping = outer_mapping.compose(&Mapping::lambda_close()).compose(inner_mapping);
					};
				}
			},
			_ => todo!(),
		},
		Term::ReplicatorA { mapping, inner } => match inner.extract() {},
		Term::ReplicatorB { mapping, inner } => match inner.extract() {},
		Term::Selector { .. } => return,
		Term::Nominal(_) => return,
	}
}
