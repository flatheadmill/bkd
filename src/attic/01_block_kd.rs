use std::cmp::Ordering;
use std::f64::INFINITY;

/// A k-dimensional point
#[derive(Clone, Copy, Debug)]
pub struct Point<const K: usize> {
    coords: [f64; K],
}

impl<const K: usize> Point<K> {
    fn distance_squared(&self, other: &Self) -> f64 {
        self.coords.iter().zip(other.coords.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum()
    }
}

/// A block in the kd-tree, containing a fixed number of points
const BLOCK_SIZE: usize = 8;
#[derive(Debug)]
pub struct Block<const K: usize> {
    points: Vec<Point<K>>,  // Stores up to BLOCK_SIZE points
    left: Option<Box<Block<K>>>,
    right: Option<Box<Block<K>>>,
    split_dim: usize,  // Dimension along which the split was made
    split_val: f64,  // Median value used for splitting
}

impl<const K: usize> Block<K> {
    pub fn new(mut points: Vec<Point<K>>, depth: usize) -> Self {
        if points.is_empty() {
            panic!("Cannot create block with zero points");
        }

        let split_dim = depth % K;
        points.sort_by(|a, b| a.coords[split_dim].partial_cmp(&b.coords[split_dim]).unwrap_or(Ordering::Equal));
        let median_idx = points.len() / 2;
        let split_val = points[median_idx].coords[split_dim];

        let (left_points, right_points) = points.split_at(median_idx);
        let left = if !left_points.is_empty() {
            Some(Box::new(Block::new(left_points.to_vec(), depth + 1)))
        } else {
            None
        };
        let right = if right_points.len() > 1 {
            Some(Box::new(Block::new(right_points[1..].to_vec(), depth + 1)))
        } else {
            None
        };

        Self {
            points: right_points.to_vec(),
            left,
            right,
            split_dim,
            split_val,
        }
    }

    /// Nearest neighbor search
    pub fn nearest_neighbor(&self, target: &Point<K>, best: &mut Option<(f64, Point<K>)>) {
        for &point in &self.points {
            let dist_sq = point.distance_squared(target);
            if best.is_none() || dist_sq < best.unwrap().0 {
                *best = Some((dist_sq, point));
            }
        }

        let go_left = target.coords[self.split_dim] < self.split_val;
        if go_left {
            if let Some(ref left) = self.left {
                left.nearest_neighbor(target, best);
            }
        } else {
            if let Some(ref right) = self.right {
                right.nearest_neighbor(target, best);
            }
        }
    }
}

/// Example usage
fn main() {
    let points = vec![
        Point { coords: [2.0, 3.0] },
        Point { coords: [5.0, 4.0] },
        Point { coords: [9.0, 6.0] },
        Point { coords: [4.0, 7.0] },
        Point { coords: [8.0, 1.0] },
        Point { coords: [7.0, 2.0] },
    ];
    
    let kd_tree = Block::<2>::new(points, 0);
    let target = Point { coords: [6.0, 3.0] };
    let mut best = None;
    kd_tree.nearest_neighbor(&target, &mut best);
    println!("Nearest neighbor: {:?}", best.unwrap().1);
}
