use crate::arena::{Arena, Node, NodeId};
use crate::tree::KDTree;

impl KDTree {
    pub fn print(&self) {
        fn print_node(arena: &Arena, id: NodeId, depth: usize) {
            let indent = "  ".repeat(depth);
            match arena.get(id) {
                Node::Leaf(points) => {
                    println!("{}Leaf: {:?}", indent, points);
                }
                Node::Internal {
                    axis,
                    split_value,
                    left,
                    right,
                } => {
                    println!(
                        "{}Internal: axis={}, split={:.2}",
                        indent, axis, split_value
                    );
                    print_node(arena, *left, depth + 1);
                    print_node(arena, *right, depth + 1);
                }
            }
        }

        if let Some(root) = self.root {
            println!("Block KD-Tree:");
            print_node(&self.arena, root, 0);
        } else {
            println!("Empty tree.");
        }
    }
}
