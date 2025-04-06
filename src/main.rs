mod tree;
mod arena;
mod print;

use tree::KDTree;
use tree::Point;

fn main() {
    let mut tree = KDTree::new();
    let points = vec![
        Point { x: 2.0, y: 3.0 },
        Point { x: 5.0, y: 4.0 },
        Point { x: 9.0, y: 6.0 },
        Point { x: 4.0, y: 7.0 },
        Point { x: 8.0, y: 1.0 },
        Point { x: 7.0, y: 2.0 },
    ];

    for p in points {
        tree.insert(p);
    }

    tree.print();
}
