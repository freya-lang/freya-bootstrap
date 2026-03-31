use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;

use super::Nominal;
use super::mapping::Mapping;

#[derive(Clone)]
pub(crate) enum Term {
	Lambda {
		mapping: Mapping,
		body: TermLink,
		argument: Option<TermLink>,
	},
	Binding {
		mapping: Mapping,
	},
	Application {
		mapping: Mapping,
		function: TermLink,
		argument: TermLink,
	},
	ReplicatorA {
		mapping: Mapping,
		inner: Rc<TermLink>,
	},
	ReplicatorB {
		mapping: Mapping,
		inner: Rc<TermLink>,
	},
	Selector {
		mapping: Mapping,
		side_a: TermLink,
		side_b: TermLink,
	},
	Nominal(Nominal),
}

#[derive(Clone)]
pub(crate) struct TermLink(Rc<RefCell<Term>>);

impl Term {
	pub(crate) fn mapping_mut(&mut self) -> Option<&mut Mapping> {
		match self {
			Term::Lambda { mapping, .. } => Some(mapping),
			Term::Binding { mapping } => Some(mapping),
			Term::Application { mapping, .. } => Some(mapping),
			Term::ReplicatorA { mapping, .. } => Some(mapping),
			Term::ReplicatorB { mapping, .. } => Some(mapping),
			Term::Selector { mapping, .. } => Some(mapping),
			Term::Nominal(_) => None,
		}
	}
}

impl TermLink {
	pub(crate) fn extract(&self) -> Term {
		self.0.borrow().clone()
	}

	pub(crate) fn modify(&self) -> impl DerefMut<Target = Term> {
		self.0.borrow_mut()
	}
}
