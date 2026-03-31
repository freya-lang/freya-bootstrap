use std::iter::from_fn;

#[derive(Clone)]
pub(crate) struct Mapping {
	pub(crate) joining_scopes: Vec<usize>,
	pub(crate) opening_scopes: Vec<usize>,
	pub(crate) closing_scopes: Vec<usize>,
}

impl Mapping {
	pub(crate) fn stack(above: &Mapping, below: &Mapping) -> Mapping {
		let mut i = 0;

		

		todo!()
	}

	pub(crate) fn lambda_top() -> Mapping {
		Mapping {
			joining_scopes: Vec::new(),
			opening_scopes: Vec::new(),
			closing_scopes: vec![0],
		}
	}

	pub(crate) fn lambda_bottom() -> Mapping {
		Mapping {
			joining_scopes: Vec::new(),
			opening_scopes: vec![0],
			closing_scopes: Vec::new(),
		}
	}

	fn trace_through_above(&self) -> impl Iterator<Item = ()> {
		let mut i = 0;
		let mut joining_i = 0;
		let mut opening_i = 0;
		let mut closing_i = 0;

		from_fn(|| {
			if i == self.opening_scopes[opening_i] {

			}

			todo!()
		})
	}
}
