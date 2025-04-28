mod trait_impls;

use std::cell::Cell;
use std::rc::Rc;

trait Signature: Sized {
	type NodeData;

	fn num_auxiliary_ports(node_data: &Self::NodeData) -> usize;
	fn link(
		left_node_data: &Self::NodeData,
		right_node_data: &Self::NodeData,
		left_ports: &[Port<Self::NodeData>],
		right_ports: &[Port<Self::NodeData>],
	);
}

struct PortCell<T>(Cell<Option<Port<T>>>);

struct DataNodeInner<T> {
	data: T,
	principal_port: PortCell<T>,
	auxiliary_ports: Box<[PortCell<T>]>,
}

struct WireNodeInner<T> {
	side_a: PortCell<T>,
	side_b: PortCell<T>,
}

struct OutputNodeInner<T> {
	connection: PortCell<T>,
}

struct DataNode<T>(Rc<DataNodeInner<T>>);
struct WireNode<T>(Rc<WireNodeInner<T>>);
struct OutputNode<T>(Rc<OutputNodeInner<T>>);

enum Node<T> {
	DataNode(DataNode<T>),
	WireNode(WireNode<T>),
	OutputNode(OutputNode<T>),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DataPortType {
	Principal,
	Auxiliary(usize),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum WireSide {
	SideA,
	SideB,
}

enum Port<T> {
	DataPort { node: DataNode<T>, port_type: DataPortType },
	WirePort { node: WireNode<T>, side: WireSide },
	OutputPort { node: OutputNode<T> },
}

impl<T> DataNode<T> {
	fn new(inner: DataNodeInner<T>) -> Self {
		Self(Rc::new(inner))
	}

	fn port(&self, port_type: DataPortType) -> Port<T> {
		Port::DataPort {
			node: self.clone(),
			port_type,
		}
	}

	fn principal(&self) -> Port<T> {
		self.port(DataPortType::Principal)
	}

	fn auxiliary(&self, index: usize) -> Port<T> {
		self.port(DataPortType::Auxiliary(index))
	}

	fn auxiliary_ports(&self) -> impl Iterator<Item = Port<T>> {
		(0 .. self.0.auxiliary_ports.len()).map(|n| self.auxiliary(n))
	}
}

impl<T> WireNode<T> {
	fn new(inner: WireNodeInner<T>) -> Self {
		Self(Rc::new(inner))
	}

	fn port(&self, side: WireSide) -> Port<T> {
		Port::WirePort {
			node: self.clone(),
			side,
		}
	}

	fn side_a(&self) -> Port<T> {
		self.port(WireSide::SideA)
	}

	fn side_b(&self) -> Port<T> {
		self.port(WireSide::SideB)
	}
}

impl<T> OutputNode<T> {
	fn new(inner: OutputNodeInner<T>) -> Self {
		Self(Rc::new(inner))
	}
}

impl<T> PortCell<T> {
	const fn new() -> Self {
		Self(Cell::new(None))
	}
}

impl<T> Port<T> {
	fn node(&self) -> Node<T> {
		match self {
			Self::DataPort { node, .. } => Node::DataNode(node.clone()),
			Self::WirePort { node, .. } => Node::WireNode(node.clone()),
			Self::OutputPort { node, .. } => Node::OutputNode(node.clone()),
		}
	}

	fn cell(&self) -> &PortCell<T> {
		match self {
			Self::DataPort { node, port_type } => match *port_type {
				DataPortType::Principal => &node.0.principal_port,
				DataPortType::Auxiliary(n) => &node.0.auxiliary_ports[n],
			},
			Self::WirePort { node, side } => match *side {
				WireSide::SideA => &node.0.side_a,
				WireSide::SideB => &node.0.side_b,
			},
			Self::OutputPort { node } => &node.0.connection,
		}
	}

	fn get_linked(&self) -> Option<Self> {
		let out = self.take_linked();
		self.set_linked(out.clone());

		out
	}

	fn set_linked(&self, value: Option<Self>) {
		self.cell().0.set(value);
	}

	fn take_linked(&self) -> Option<Self> {
		self.cell().0.take()
	}
}

fn link_pair<T>(left: &Port<T>, right: &Port<T>) {
	// both ports should previously be disconnected
	assert!(left.get_linked().is_none());
	assert!(right.get_linked().is_none());

	left.set_linked(Some(right.clone()));
	right.set_linked(Some(left.clone()));
}

fn link_many_pairs<T>(pairs: &[(&Port<T>, &Port<T>)]) {
	for (left, right) in pairs {
		link_pair(left, right);
	}
}

fn unlink_pair<T>(left: &Port<T>, right: &Port<T>) {
	assert!(is_pair_linked(left, right));

	left.set_linked(None);
	right.set_linked(None);
}

fn unlink_many_pairs<T>(pairs: &[(&Port<T>, &Port<T>)]) {
	for (left, right) in pairs {
		unlink_pair(left, right);
	}
}

fn is_pair_linked<T>(left: &Port<T>, right: &Port<T>) -> bool {
	let Some(left_linked) = left.get_linked() else {
		return false;
	};
	let Some(right_linked) = right.get_linked() else {
		return false;
	};

	left_linked == *right && right_linked == *left
}

fn contract_wire_node<T>(wire_node: &WireNode<T>) {
	let left = wire_node
		.side_a()
		.take_linked()
		.expect("wire nodes should be connected");
	let right = wire_node
		.side_b()
		.take_linked()
		.expect("wire nodes should be connected");

	let left_wire_backlink = left.take_linked().expect("backlinks should be connected");
	let right_wire_backlink = right.take_linked().expect("backlinks should be connected");

	assert!(left_wire_backlink == wire_node.side_a());
	assert!(right_wire_backlink == wire_node.side_b());

	link_pair(&left, &right);
}

fn insert_wire_node<T>(port: &Port<T>) -> WireNode<T> {
	let linked = &port.take_linked().expect("port should be connected");
	let backlink = &linked.take_linked().expect("backlink should exist");

	assert!(port == backlink);

	let wire = WireNode::new(WireNodeInner {
		side_a: PortCell::new(),
		side_b: PortCell::new(),
	});

	link_pair(port, &wire.side_a());
	link_pair(linked, &wire.side_b());

	wire
}

fn new_node<T, S: Signature<NodeData = T>>(data: T) -> DataNode<T> {
	let num_aux_ports = S::num_auxiliary_ports(&data);
	let mut auxiliary_ports = Vec::with_capacity(num_aux_ports);

	for _ in 0 .. num_aux_ports {
		auxiliary_ports.push(PortCell::new());
	}

	DataNode::new(DataNodeInner {
		data,
		principal_port: PortCell::new(),
		auxiliary_ports: auxiliary_ports.into_boxed_slice(),
	})
}

fn retract<T>(port: &Port<T>) -> Port<T> {
	let out = port.get_linked().unwrap();
	unlink_pair(port, &out);

	out
}

fn interact<T, S: Signature<NodeData = T>>(node_a: &DataNode<T>, node_b: &DataNode<T>) {
	unlink_pair(&node_a.principal(), &node_b.principal());

	let mut wires = Vec::new();

	for port in node_a.auxiliary_ports().chain(node_b.auxiliary_ports()) {
		wires.push(insert_wire_node(&port));
	}

	let mut linked_a = Vec::new();
	let mut linked_b = Vec::new();

	for port in node_a.auxiliary_ports() {
		linked_a.push(retract(&port));
	}
	for port in node_b.auxiliary_ports() {
		linked_b.push(retract(&port));
	}

	S::link(&node_a.0.data, &node_b.0.data, &linked_a, &linked_b);

	for wire in wires {
		contract_wire_node(&wire);
	}
}
