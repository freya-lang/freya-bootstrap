use std::rc::Rc;

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum Value {
	Universe(Universe),
	Intrinsic {
		head: Intrinsic,
		arguments: Vec<Value>,
	},
	Binding {
		level: usize,
	},
	Lambda {
		inner: Box<Value>,
	},
	PiType {
		parameter_type: Box<Value>,
		inner: Box<Value>,
		is_propositional: bool,
	},
	Witness,
	Application {
		left: Box<Value>,
		right: Box<Value>,
	},
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Universe {
	Prop,
	Set { level: usize },
}

#[derive(Clone)]
pub(crate) struct Intrinsic(pub Rc<IntrinsicData>);

pub(crate) struct IntrinsicData {}

pub(crate) enum LoweredExpr {
	Lambda {
		parameter_type: Box<LoweredExpr>,
		inner: Box<LoweredExpr>,
		vtp_store: VtpStore,
	},
	PiType {
		parameter_type: Box<LoweredExpr>,
		return_type: Box<LoweredExpr>,
		vtp_store: VtpStore,
	},
	Application {
		left: Box<LoweredExpr>,
		right: Box<LoweredExpr>,
		vtp_store: VtpStore,
	},
	Binding {
		level: usize,
		vtp_store: VtpStore,
	},
	External {
		vtp: Vtp,
		vtp_store: VtpStore,
	},
}

#[derive(Clone)]
pub(crate) struct Vtp {
	pub value: Value,
	pub type_of: Value,
	pub is_propositional: bool,
}

#[derive(Clone)]
pub(crate) struct Tp {
	pub inner: Value,
	pub is_propositional: bool,
}

pub(crate) struct VtpStore {
	pub value: Option<Value>,
	pub tp: Option<Tp>,
}

pub(crate) enum BindingStack<'a, T> {
	Empty,
	HasValue { value: &'a T, previous: &'a Self },
}
