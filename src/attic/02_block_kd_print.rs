const BLOCK_SIZE: usize = 4;

#[derive(Debug, Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Debug)]
enum Node {
    Leaf(Vec<Point>),
    Internal {
        axis: usize,
        split_value: f64,
        left: Box<Node>,
        right: Box<Node>,
    },
}

#[derive(Debug)]
pub struct KDTree {
    root: Option<Box<Node>>,
}

impl KDTree {
    pub fn new() -> Self {
        KDTree { root: None }
    }

    pub fn insert(&mut self, point: Point) {
        match self.root {
            Some(ref mut root) => Self::insert_rec(root, point, 0),
            None => {
                self.root = Some(Box::new(Node::Leaf(vec![point])));
            }
        }
    }

    fn insert_rec(node: &mut Box<Node>, point: Point, depth: usize) {
        match **node {
            Node::Leaf(ref mut points) => {
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
    
                    let (left_points, right_points): (Vec<_>, Vec<_>) = points
                        .drain(..)
                        .partition(|p| {
                            if axis == 0 {
                                p.x < split_value
                            } else {
                                p.y < split_value
                            }
                        });
    
                    **node = Node::Internal {
                        axis,
                        split_value,
                        left: Box::new(Node::Leaf(left_points)),
                        right: Box::new(Node::Leaf(right_points)),
                    };
                }
            }
    
            Node::Internal {
                axis,
                split_value,
                ref mut left,
                ref mut right,
            } => {
                let target = if (axis == 0 && point.x < split_value)
                    || (axis == 1 && point.y < split_value)
                {
                    left
                } else {
                    right
                };
    
                Self::insert_rec(target, point, depth + 1);
            }
        }
    }
    
}

impl KDTree {
    pub fn print(&self) {
        fn print_node(node: &Box<Node>, depth: usize) {
            let indent = "  ".repeat(depth);
            match **node {
                Node::Leaf(ref points) => {
                    println!("{}Leaf: {:?}", indent, points);
                }
                Node::Internal {
                    axis,
                    split_value,
                    ref left,
                    ref right,
                } => {
                    println!(
                        "{}Internal: axis={}, split={:.1}",
                        indent, axis, split_value
                    );
                    print_node(left, depth + 1);
                    print_node(right, depth + 1);
                }
            }
        }

        if let Some(ref node) = self.root {
            println!("Block KD-Tree:");
            print_node(node, 0);
        } else {
            println!("Empty tree.");
        }
    }
}


fn main() {
    let mut tree = KDTree::new();
    // let points = vec![
    //     Point { x: 2.0, y: 3.0 },
    //     Point { x: 5.0, y: 4.0 },
    //     Point { x: 9.0, y: 6.0 },
    //     Point { x: 4.0, y: 7.0 },
    //     Point { x: 8.0, y: 1.0 },
    //     Point { x: 7.0, y: 2.0 },
    // ];

    let points = vec![
    Point { x: 2.0, y: 3.0 },
    Point { x: 5.0, y: 4.0 },
    Point { x: 9.0, y: 6.0 },
    Point { x: 4.0, y: 7.0 },
    Point { x: 8.0, y: 1.0 },
    Point { x: 7.0, y: 2.0 },
    Point { x: 1.0, y: 5.0 },
    Point { x: 3.0, y: 9.0 },
    Point { x: 6.0, y: 8.0 },
    Point { x: 0.0, y: 2.0 },
    Point { x: 5.5, y: 5.5 },
    Point { x: 2.5, y: 6.5 },
    Point { x: 3.5, y: 3.5 },
    Point { x: 7.5, y: 4.5 },
    Point { x: 1.5, y: 1.5 },
    Point { x: 9.5, y: 7.5 },
    Point { x: 6.5, y: 1.0 },
    Point { x: 4.5, y: 0.5 },
    Point { x: 8.5, y: 9.5 },
    Point { x: 0.5, y: 8.5 },
    Point { x: 3.3, y: 1.2 },
    Point { x: 2.2, y: 4.4 },
    Point { x: 5.1, y: 6.3 },
    Point { x: 7.8, y: 5.9 },
    Point { x: 6.7, y: 7.1 },
    Point { x: 4.4, y: 2.2 },
    Point { x: 9.9, y: 0.1 },
    Point { x: 0.9, y: 3.3 },
    Point { x: 1.1, y: 6.6 },
    Point { x: 2.9, y: 8.8 },
    Point { x: 3.7, y: 0.9 },
    Point { x: 5.9, y: 2.7 },
    Point { x: 6.2, y: 3.3 },
    Point { x: 7.3, y: 6.6 },
    Point { x: 8.4, y: 8.0 },
    Point { x: 9.1, y: 4.2 },
    Point { x: 4.1, y: 1.1 },
    Point { x: 3.9, y: 2.3 },
    Point { x: 6.8, y: 5.0 },
    Point { x: 2.6, y: 3.2 },
    Point { x: 1.4, y: 0.4 },
    Point { x: 8.9, y: 3.6 },
    Point { x: 7.1, y: 9.3 },
    Point { x: 0.7, y: 7.7 },
];

    for p in points {
        tree.insert(p);
        tree.print();
    }

    tree.print();
}

/*

let points = vec![
    Point { x: 2.0, y: 3.0 },
    Point { x: 5.0, y: 4.0 },
    Point { x: 9.0, y: 6.0 },
    Point { x: 4.0, y: 7.0 },
    Point { x: 8.0, y: 1.0 },
    Point { x: 7.0, y: 2.0 },
    Point { x: 1.0, y: 5.0 },
    Point { x: 3.0, y: 9.0 },
    Point { x: 6.0, y: 8.0 },
    Point { x: 0.0, y: 2.0 },
    Point { x: 5.5, y: 5.5 },
    Point { x: 2.5, y: 6.5 },
    Point { x: 3.5, y: 3.5 },
    Point { x: 7.5, y: 4.5 },
    Point { x: 1.5, y: 1.5 },
    Point { x: 9.5, y: 7.5 },
    Point { x: 6.5, y: 1.0 },
    Point { x: 4.5, y: 0.5 },
    Point { x: 8.5, y: 9.5 },
    Point { x: 0.5, y: 8.5 },
    Point { x: 3.3, y: 1.2 },
    Point { x: 2.2, y: 4.4 },
    Point { x: 5.1, y: 6.3 },
    Point { x: 7.8, y: 5.9 },
    Point { x: 6.7, y: 7.1 },
    Point { x: 4.4, y: 2.2 },
    Point { x: 9.9, y: 0.1 },
    Point { x: 0.9, y: 3.3 },
    Point { x: 1.1, y: 6.6 },
    Point { x: 2.9, y: 8.8 },
    Point { x: 3.7, y: 0.9 },
    Point { x: 5.9, y: 2.7 },
    Point { x: 6.2, y: 3.3 },
    Point { x: 7.3, y: 6.6 },
    Point { x: 8.4, y: 8.0 },
    Point { x: 9.1, y: 4.2 },
    Point { x: 4.1, y: 1.1 },
    Point { x: 3.9, y: 2.3 },
    Point { x: 6.8, y: 5.0 },
    Point { x: 2.6, y: 3.2 },
    Point { x: 1.4, y: 0.4 },
    Point { x: 8.9, y: 3.6 },
    Point { x: 7.1, y: 9.3 },
    Point { x: 0.7, y: 7.7 },
];

for p in points {
    tree.insert(p);
}


*/