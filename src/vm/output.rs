use super::Nominal;
use super::term::{Term, TermLink};

enum Output {
	Lambda {
		parameter: Nominal,
		body: Box<Resolvable>,
	},
	Application {
		function: Box<Resolvable>,
		argument: Box<Resolvable>,
	},
	Nominal(Nominal),
}

enum Resolvable {
	Resolved(Output),
	Unresolved(TermLink),
}

impl Resolvable {
	fn resolve(&mut self) -> &mut Output {
		match self {
			Self::Resolved(output) => output,
			Self::Unresolved(current) => {
				let mut resolution_stack = Vec::new();
				let mut current = current.clone();

				let output = loop {
					match current.extract() {
						Term::Lambda {
							mapping,
							body,
							argument,
						} => {
							if let Some(above) = resolution_stack.pop() {
								todo!();
							} else {
								let parameter = Nominal::new();

								if let Some(argument) = argument {
									*argument.modify() = Term::Nominal(parameter);
								}

								break Output::Lambda {
									parameter,
									body: Box::new(Resolvable::Unresolved(body)),
								};
							}
						},
						Term::Binding { .. } => unreachable!(),
						Term::Application { function, .. } => {
							resolution_stack.push(current);
							current = function;
						},
						Term::ReplicatorA { inner, .. } | Term::ReplicatorB { inner, .. } => {
							resolution_stack.push(current);
							current = (*inner).clone();
						},
						Term::Selector { .. } => unreachable!(),
						Term::Nominal(nominal) => {
							let mut output = Output::Nominal(nominal);

							while let Some(above) = resolution_stack.pop() {
								match above.extract() {
									Term::Lambda { .. } => unreachable!(),
									Term::Binding { .. } => unreachable!(),
									Term::Application {
										mapping,
										function,
										argument,
									} => todo!(),
									Term::ReplicatorA { mapping, inner } => todo!(),
									Term::ReplicatorB { mapping, inner } => todo!(),
									Term::Selector { .. } => unreachable!(),
									Term::Nominal(_) => unreachable!(),
								}
							}

							break output;
						},
					}
				};

				*self = Self::Resolved(output);

				match self {
					Self::Resolved(output) => output,
					Self::Unresolved(_) => unreachable!(),
				}
			},
		}
	}
}
