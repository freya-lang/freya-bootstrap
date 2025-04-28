use std::rc::Rc;

use super::{DataNode, Node, OutputNode, Port, WireNode};

// manual implementations because #[derive(Clone)] applies a T: Clone bound

impl<T> Clone for DataNode<T> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<T> Clone for WireNode<T> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<T> Clone for OutputNode<T> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<T> Clone for Port<T> {
	fn clone(&self) -> Self {
		match self {
			&Self::DataPort { ref node, port_type } => Self::DataPort {
				node: node.clone(),
				port_type,
			},
			&Self::WirePort { ref node, side } => Self::WirePort {
				node: node.clone(),
				side,
			},
			Self::OutputPort { node } => Self::OutputPort { node: node.clone() },
		}
	}
}

// manual implementations to use Rc::ptr_eq and because the derives apply T: Eq bounds

impl<T> PartialEq for DataNode<T> {
	fn eq(&self, other: &Self) -> bool {
		Rc::ptr_eq(&self.0, &other.0)
	}
}

impl<T> PartialEq for WireNode<T> {
	fn eq(&self, other: &Self) -> bool {
		Rc::ptr_eq(&self.0, &other.0)
	}
}

impl<T> PartialEq for OutputNode<T> {
	fn eq(&self, other: &Self) -> bool {
		Rc::ptr_eq(&self.0, &other.0)
	}
}

impl<T> PartialEq for Node<T> {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::DataNode(node_a), Self::DataNode(node_b)) => node_a == node_b,
			(Self::WireNode(node_a), Self::WireNode(node_b)) => node_a == node_b,
			(Self::OutputNode(node_a), Self::OutputNode(node_b)) => node_a == node_b,
			_ => false,
		}
	}
}

impl<T> PartialEq for Port<T> {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(
				Self::DataPort {
					node: node_a,
					port_type: port_type_a,
				},
				Self::DataPort {
					node: node_b,
					port_type: port_type_b,
				},
			) => node_a == node_b && port_type_a == port_type_b,
			(
				Self::WirePort {
					node: node_a,
					side: side_a,
				},
				Self::WirePort {
					node: node_b,
					side: side_b,
				},
			) => node_a == node_b && side_a == side_b,
			(Self::OutputPort { node: node_a }, Self::OutputPort { node: node_b }) => node_a == node_b,
			_ => false,
		}
	}
}

impl<T> Eq for DataNode<T> {}
impl<T> Eq for WireNode<T> {}
impl<T> Eq for OutputNode<T> {}
impl<T> Eq for Node<T> {}
impl<T> Eq for Port<T> {}
