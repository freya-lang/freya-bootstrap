mod annotate;
mod datastructures;
mod lower;
mod methods;
mod trait_impls;

use std::collections::HashMap;

use self::annotate::{get_type, get_value};
use self::datastructures::BindingStack;
pub(crate) use self::datastructures::{Intrinsic, IntrinsicData, Value};
use self::lower::lower_expr;
use crate::parser::Expr;

pub(crate) fn evaluate(expr: Expr) -> Value {
	let mut lowered = lower_expr(expr, &mut HashMap::new(), 0);

	get_type(&mut lowered, BindingStack::Empty);
	get_value(&mut lowered)
}
