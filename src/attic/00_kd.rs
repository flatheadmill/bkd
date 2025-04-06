#[derive(Debug, Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Debug)]
struct Node {
    point: Point,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
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
        self.root = Self::insert_rec(self.root.take(), point, 0);
    }

    fn insert_rec(node: Option<Box<Node>>, point: Point, depth: usize) -> Option<Box<Node>> {
        match node {
            Some(mut n) => {
                // Flip flop axes at each level
                let axis = depth % 2;
                if (axis == 0 && point.x < n.point.x) || (axis == 1 && point.y < n.point.y) {
                    n.left = Self::insert_rec(n.left.take(), point, depth + 1);
                } else {
                    n.right = Self::insert_rec(n.right.take(), point, depth + 1);
                }
                Some(n)
            }
            None => Some(Box::new(Node {
                point,
                left: None,
                right: None,
            })),
        }
    }

    pub fn print(&self) {
        fn print_node(node: &Option<Box<Node>>, depth: usize) {
            if let Some(n) = node {
                let indent = "  ".repeat(depth);
                println!("{}• ({:.1}, {:.1})", indent, n.point.x, n.point.y);
                print_node(&n.left, depth + 1);
                print_node(&n.right, depth + 1);
            }
        }

        println!("KD-Tree:");
        print_node(&self.root, 0);
    }    
}

fn main() {
    let mut tree = KDTree::new();
    tree.insert(Point { x: 2.0, y: 3.0 });
    tree.insert(Point { x: 5.0, y: 4.0 });
    tree.insert(Point { x: 9.0, y: 6.0 });
    tree.insert(Point { x: 4.0, y: 7.0 });
    tree.insert(Point { x: 8.0, y: 1.0 });
    tree.insert(Point { x: 7.0, y: 2.0 });

    // println!("{:?}", tree);

    tree.print();
}