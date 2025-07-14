use crate::evaluator::datastructures::{BindingStack, LoweredExpr, Value, VtpStore};

impl Value {
	pub(crate) fn normalize(self) -> Self {
		match self {
			Value::Intrinsic { head, arguments } => Value::Intrinsic {
				head,
				arguments: arguments.into_iter().map(|x| x.normalize()).collect(),
			},
			Value::Lambda { inner } => Value::Lambda {
				inner: Box::new(inner.normalize()),
			},
			Value::PiType {
				parameter_type,
				inner,
				is_propositional,
			} => Value::PiType {
				parameter_type: Box::new(parameter_type.normalize()),
				inner: Box::new(inner.normalize()),
				is_propositional,
			},
			Value::Application { left, right } => {
				let left = left.normalize();

				match left {
					Value::Intrinsic { head, mut arguments } => {
						arguments.push(right.normalize());

						Value::Intrinsic { head, arguments }
					},
					Value::Lambda { inner } => inner.substitute(*right),
					Value::Witness => Value::Witness,
					_ => Value::Application {
						left: Box::new(left),
						right: Box::new(right.normalize()),
					},
				}
			},
			x @ _ => x,
		}
	}

	fn add_to_bindings(&self, amount: usize) -> Value {
		match self {
			&Value::Intrinsic {
				ref head,
				ref arguments,
			} => Value::Intrinsic {
				head: head.clone(),
				arguments: arguments.iter().map(|x| x.add_to_bindings(amount)).collect(),
			},
			&Value::Binding { level } => Value::Binding {
				level: level.checked_add(amount).unwrap(),
			},
			Value::Lambda { inner } => Value::Lambda {
				inner: Box::new(inner.add_to_bindings(amount)),
			},
			&Value::PiType {
				ref parameter_type,
				ref inner,
				is_propositional,
			} => Value::PiType {
				parameter_type: Box::new(parameter_type.add_to_bindings(amount)),
				inner: Box::new(inner.add_to_bindings(amount)),
				is_propositional,
			},
			Value::Application { left, right } => Value::Application {
				left: Box::new(left.add_to_bindings(amount)),
				right: Box::new(right.add_to_bindings(amount)),
			},
			x @ _ => x.clone(),
		}
	}

	pub(crate) fn substitute(self, argument: Value) -> Value {
		self.substitute_n(&argument, 0).normalize()
	}

	fn substitute_n(self, argument: &Value, for_index: usize) -> Value {
		match self {
			Value::Intrinsic { head, arguments } => Value::Intrinsic {
				head,
				arguments: arguments
					.into_iter()
					.map(|x| x.substitute_n(argument, for_index))
					.collect(),
			},
			Value::Binding { level, .. } if level == for_index => argument.add_to_bindings(level),
			Value::Binding { level, .. } if level > for_index => argument.add_to_bindings(level - 1),
			Value::Lambda { inner } => Value::Lambda {
				inner: Box::new(inner.substitute_n(argument, for_index.checked_add(1).unwrap())),
			},
			Value::PiType {
				parameter_type,
				inner,
				is_propositional,
			} => Value::PiType {
				parameter_type: Box::new(parameter_type.substitute_n(argument, for_index)),
				inner: Box::new(inner.substitute_n(argument, for_index.checked_add(1).unwrap())),
				is_propositional,
			},
			Value::Application { left, right } => Value::Application {
				left: Box::new(left.substitute_n(argument, for_index)),
				right: Box::new(right.substitute_n(argument, for_index)),
			},
			x @ _ => x,
		}
	}
}

impl LoweredExpr {
	pub(crate) fn vtp_store_im(&self) -> &VtpStore {
		match self {
			Self::Lambda { vtp_store, .. } => vtp_store,
			Self::PiType { vtp_store, .. } => vtp_store,
			Self::Application { vtp_store, .. } => vtp_store,
			Self::Binding { vtp_store, .. } => vtp_store,
			Self::External { vtp_store, .. } => vtp_store,
		}
	}

	pub(crate) fn vtp_store(&mut self) -> &mut VtpStore {
		match self {
			Self::Lambda { vtp_store, .. } => vtp_store,
			Self::PiType { vtp_store, .. } => vtp_store,
			Self::Application { vtp_store, .. } => vtp_store,
			Self::Binding { vtp_store, .. } => vtp_store,
			Self::External { vtp_store, .. } => vtp_store,
		}
	}
}

impl VtpStore {
	pub(crate) fn new() -> Self {
		Self { value: None, tp: None }
	}
}

impl<'a, T> BindingStack<'a, T> {
	pub(crate) fn open(self) -> (&'a T, Self) {
		match self {
			Self::Empty => panic!("attempting to open BindingStack node that was empty"),
			Self::HasValue { value, previous } => (value, *previous),
		}
	}

	pub(crate) fn nth_upward(mut self, index: usize) -> Self {
		for _ in 0 .. index {
			self = self.open().1;
		}

		self
	}

	pub(crate) fn add_value<'b>(&'b self, value: &'b T) -> BindingStack<'b, T> {
		BindingStack::HasValue { value, previous: self }
	}
}
