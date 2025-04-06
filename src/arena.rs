use crate::tree::Point;

pub type NodeId = usize;

#[derive(Debug)]
pub enum Node {
    Leaf(Vec<Point>),
    Internal {
        axis: usize,
        split_value: f64,
        left: NodeId,
        right: NodeId,
    },
}

#[derive(Debug)]
pub struct Arena {
    pub nodes: Vec<Node>,
}

impl Arena {
    pub fn new() -> Self {
        Arena { nodes: Vec::new() }
    }

    pub fn alloc(&mut self, node: Node) -> NodeId {
        let id = self.nodes.len();
        self.nodes.push(node);
        id
    }

    pub fn get(&self, id: NodeId) -> &Node {
        &self.nodes[id]
    }

    pub fn get_mut(&mut self, id: NodeId) -> &mut Node {
        &mut self.nodes[id]
    }
}
