use crate::evaluator::datastructures::{BindingStack, LoweredExpr, Tp, Universe, Value};

fn pi_universe(parameter_universe: Universe, return_universe: Universe) -> Universe {
	match return_universe {
		Universe::Prop => Universe::Prop,
		Universe::Set { level: level_a } => match parameter_universe {
			Universe::Prop => Universe::Set { level: level_a },
			Universe::Set { level: level_b } => Universe::Set {
				level: level_a.max(level_b),
			},
		},
	}
}

pub(crate) fn get_value(expr: &mut LoweredExpr) -> Value {
	match &expr.vtp_store().value {
		None => {
			let value = if is_propositional(expr) {
				Value::Witness
			} else {
				get_value_uncached(expr)
			};

			expr.vtp_store().value = Some(value.clone());

			value
		},
		Some(out) => out.clone(),
	}
}

fn get_tp(expr: &mut LoweredExpr, bindings: BindingStack<'_, Tp>) -> Tp {
	match &expr.vtp_store().tp {
		None => {
			let out = get_tp_uncached(expr, bindings);
			expr.vtp_store().tp = Some(out.clone());

			out
		},
		Some(out) => out.clone(),
	}
}

pub(crate) fn get_type(expr: &mut LoweredExpr, bindings: BindingStack<'_, Tp>) -> Value {
	match &expr.vtp_store().tp {
		None => {
			let tp = get_tp_uncached(expr, bindings);
			let out = tp.inner.clone();
			expr.vtp_store().tp = Some(tp);

			out
		},
		Some(tp) => tp.inner.clone(),
	}
}

fn is_propositional(expr: &LoweredExpr) -> bool {
	expr.vtp_store_im().tp.as_ref().unwrap().is_propositional
}

fn assume_type_is_known(expr: &LoweredExpr) -> &Value {
	&expr.vtp_store_im().tp.as_ref().unwrap().inner
}

fn get_value_uncached(expr: &mut LoweredExpr) -> Value {
	match expr {
		LoweredExpr::Lambda { inner, .. } => Value::Lambda {
			inner: Box::new(get_value(inner)),
		},
		LoweredExpr::PiType {
			parameter_type,
			return_type,
			..
		} => {
			let parameter_type = get_value(parameter_type);

			let &Value::Universe(return_universe) = assume_type_is_known(return_type) else {
				unreachable!("should be ruled out by typechecking");
			};

			let return_type = get_value(return_type);

			Value::PiType {
				parameter_type: Box::new(parameter_type),
				inner: Box::new(return_type),
				is_propositional: matches!(return_universe, Universe::Prop),
			}
		},
		LoweredExpr::Application { left, right, .. } => Value::Application {
			left: Box::new(get_value(left)),
			right: Box::new(get_value(right)),
		}
		.normalize(),
		&mut LoweredExpr::Binding { level, .. } => Value::Binding { level },
		LoweredExpr::External { vtp, .. } => vtp.value.clone(),
	}
}

fn get_tp_uncached(expr: &mut LoweredExpr, bindings: BindingStack<'_, Tp>) -> Tp {
	match expr {
		LoweredExpr::Lambda {
			parameter_type, inner, ..
		} => {
			let parameter_universe = get_type(parameter_type, bindings);

			let Value::Universe(parameter_universe) = parameter_universe else {
				panic!("the type of a parameter type must be a universe");
			};

			let parameter_type = get_value(parameter_type);

			let Tp {
				inner: inner_type,
				is_propositional,
			} = get_tp(
				inner,
				bindings.add_value(&Tp {
					inner: parameter_type.clone(),
					is_propositional: matches!(parameter_universe, Universe::Prop),
				}),
			);

			Tp {
				inner: Value::PiType {
					parameter_type: Box::new(parameter_type),
					inner: Box::new(inner_type),
					is_propositional,
				},
				is_propositional,
			}
		},
		LoweredExpr::PiType {
			parameter_type,
			return_type,
			..
		} => {
			let parameter_universe = get_type(parameter_type, bindings);

			let Value::Universe(parameter_universe) = parameter_universe else {
				panic!("the type of a parameter type must be a universe");
			};

			let parameter_type = get_value(parameter_type);

			let return_universe = get_type(
				return_type,
				bindings.add_value(&Tp {
					inner: parameter_type.clone(),
					is_propositional: matches!(parameter_universe, Universe::Prop),
				}),
			);

			let Value::Universe(return_universe) = return_universe else {
				panic!("the type of a return type must be a universe");
			};

			let out_universe = pi_universe(parameter_universe, return_universe);

			Tp {
				inner: Value::Universe(out_universe),
				is_propositional: false,
			}
		},
		LoweredExpr::Application { left, right, .. } => {
			let left_type = get_type(left, bindings);

			let Value::PiType {
				parameter_type,
				inner,
				is_propositional,
			} = left_type
			else {
				panic!("left side of an application must be a pi type");
			};

			let right_type = get_type(right, bindings);

			if *parameter_type != right_type {
				panic!("type mismatch in application");
			}

			let right_value = get_value(right);

			Tp {
				inner: inner.substitute(right_value),
				is_propositional,
			}
		},
		LoweredExpr::Binding { level, .. } => bindings.nth_upward(*level).open().0.clone(),
		LoweredExpr::External { vtp, .. } => Tp {
			inner: vtp.type_of.clone(),
			is_propositional: vtp.is_propositional,
		},
	}
}
