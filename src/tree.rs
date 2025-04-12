use crate::arena::{Arena, Node, NodeId};

pub const BLOCK_SIZE: usize = 4;

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

pub struct KDTree {
    pub root: Option<NodeId>,
    pub arena: Arena,
}

impl KDTree {
    pub fn new() -> Self {
        KDTree {
            root: None,
            arena: Arena::new(),
        }
    }

    pub fn insert(&mut self, point: Point) {
        match self.root {
            None => {
                self.root = Some(self.arena.alloc(Node::Leaf(vec![point])));
            }
            Some(id) => {
                Self::insert_rec(id, point, 0, &mut self.arena);
            }
        }
    }

    fn insert_rec(id: NodeId, point: Point, depth: usize, arena: &mut Arena) {
        match arena.get_mut(id) {
            Node::Leaf(points) => {
                points.push(point);
                if points.len() > BLOCK_SIZE {
                    let axis = depth % 2;
                    points.sort_by(|a, b| {
                        let key = |p: &Point| if axis == 0 { p.x } else { p.y };
                        key(a).partial_cmp(&key(b)).unwrap()
                    });

                    let median = points.len() / 2;
                    let split_value = if axis == 0 {
                        points[median].x
                    } else {
                        points[median].y
                    };

                    /*
                     * Note to self: 
                     *  using `drain()` here to
                     *      - empty the vector `points` (can't be *borrowing* points)
                     *      - take ownership of every point in the vector
                     *      - yield an iterator
                     */

                    let (left_points, right_points): (Vec<_>, Vec<_>) =
                        points.drain(..).partition(|p| {
                            if axis == 0 {
                                p.x < split_value
                            } else {
                                p.y < split_value
                            }
                        });

                    let left_id = arena.alloc(Node::Leaf(left_points));
                    let right_id = arena.alloc(Node::Leaf(right_points));

                    *arena.get_mut(id) = Node::Internal {
                        axis,
                        split_value,
                        left: left_id,
                        right: right_id,
                    };
                }
            }

            Node::Internal {
                axis,
                split_value,
                left,
                right,
            } => {
                let target_id = if (*axis == 0 && point.x < *split_value)
                    || (*axis == 1 && point.y < *split_value)
                {
                    *left
                } else {
                    *right
                };

                Self::insert_rec(target_id, point, depth + 1, arena);
            }
        }
    }

}
