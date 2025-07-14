use std::rc::Rc;

use crate::evaluator::datastructures::{BindingStack, Intrinsic};

impl PartialEq for Intrinsic {
	fn eq(&self, other: &Self) -> bool {
		Rc::ptr_eq(&self.0, &other.0)
	}
}

impl Eq for Intrinsic {}

impl<T> Clone for BindingStack<'_, T> {
	fn clone(&self) -> Self {
		match self {
			Self::Empty => Self::Empty,
			Self::HasValue { value, previous } => Self::HasValue { value, previous },
		}
	}
}

impl<T> Copy for BindingStack<'_, T> {}
