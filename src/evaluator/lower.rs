use std::collections::HashMap;

use crate::evaluator::datastructures::{LoweredExpr, Universe, Value, Vtp, VtpStore};
use crate::parser::{BindingData, Expr, ExprData};

pub(crate) fn lower_expr(expr: Expr, bindings: &mut HashMap<String, usize>, mut level: usize) -> LoweredExpr {
	match expr.data {
		ExprData::FnLowercase {
			args,
			return_type,
			body,
		} => {
			let mut shadows = Vec::new();
			let mut param_types = Vec::new();

			for arg in args {
				let arg = arg.data;

				let binding = arg.binding;
				let ascribed_type = arg.ascribed_type.expect("all lambdas should have ascribed types");
				let ascribed_type = lower_expr(ascribed_type, bindings, level);

				param_types.push(ascribed_type);

				if let BindingData::Identifier { binding_name } = binding.data {
					let previous_value = bindings.insert(binding_name.clone(), level);
					shadows.push((binding_name, previous_value));
				}
				level = level.checked_add(1).unwrap();
			}

			if !return_type.is_none() {
				panic!("return types are expected to not be provided");
			}

			let body = lower_expr(*body, bindings, level);

			for (binding_name, previous_value) in shadows.into_iter().rev() {
				if let Some(previous_value) = previous_value {
					bindings.insert(binding_name, previous_value);
				}
			}

			let mut out = body;

			for param_type in param_types.into_iter().rev() {
				out = LoweredExpr::Lambda {
					parameter_type: Box::new(param_type),
					inner: Box::new(out),
					vtp_store: VtpStore::new(),
				};
			}

			out
		},
		ExprData::FnUppercase { args, return_type } => {
			let mut shadows = Vec::new();
			let mut param_types = Vec::new();

			for arg in args {
				let arg = arg.data;

				let binding = arg.binding;
				let ascribed_type = arg.ascribed_type.expect("pi type bindings should have ascribed types");
				let ascribed_type = lower_expr(ascribed_type, bindings, level);

				param_types.push(ascribed_type);

				if let BindingData::Identifier { binding_name } = binding.data {
					let previous_value = bindings.insert(binding_name.clone(), level);
					shadows.push((binding_name, previous_value));
				}
				level = level.checked_add(1).unwrap();
			}

			let return_type = lower_expr(*return_type, bindings, level);

			for (binding_name, previous_value) in shadows.into_iter().rev() {
				if let Some(previous_value) = previous_value {
					bindings.insert(binding_name, previous_value);
				}
			}

			let mut out = return_type;

			for param_type in param_types.into_iter().rev() {
				out = LoweredExpr::PiType {
					parameter_type: Box::new(param_type),
					return_type: Box::new(out),
					vtp_store: VtpStore::new(),
				};
			}

			out
		},
		ExprData::Block { .. } => unimplemented!(),
		ExprData::Grouping { inner } => lower_expr(*inner, bindings, level),
		ExprData::Application { left, right } => {
			let left = lower_expr(*left, bindings, level);
			let right = lower_expr(*right, bindings, level);

			LoweredExpr::Application {
				left: Box::new(left),
				right: Box::new(right),
				vtp_store: VtpStore::new(),
			}
		},
		ExprData::Value { mut path } => {
			if path.len() != 1 {
				unimplemented!();
			}

			let binding_name = path.pop().unwrap();

			let referenced_level = bindings.get(&binding_name).expect("undefined binding");

			LoweredExpr::Binding {
				level: level - referenced_level,
				vtp_store: VtpStore::new(),
			}
		},
		ExprData::Set { level } => LoweredExpr::External {
			vtp: Vtp {
				value: Value::Universe(Universe::Set { level }),
				type_of: Value::Universe(Universe::Set {
					level: level.checked_add(1).unwrap(),
				}),
				is_propositional: false,
			},
			vtp_store: VtpStore::new(),
		},
		ExprData::Prop => LoweredExpr::External {
			vtp: Vtp {
				value: Value::Universe(Universe::Prop),
				type_of: Value::Universe(Universe::Set { level: 0 }),
				is_propositional: false,
			},
			vtp_store: VtpStore::new(),
		},
	}
}
