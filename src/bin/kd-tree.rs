// Core trait for any type that can be used in a KD-tree
// Provides dimensional access for tree algorithms (splitting, traversal, etc.)
trait Point {
    fn get_dimension(&self, dim: usize) -> f64;
    fn dimensions(&self) -> usize;
}

// Extended trait for spatial range queries (bounding box searches, etc.)
// Separates spatial operations from basic KD-tree operations
trait SpatialPoint: Point {
    fn is_within(&self, query: &Self) -> bool;
    fn overlaps(&self, query: &Self) -> bool;
}

// 4-dimensional bounding box for spatial indexing
// Represents a rectangular region in 2D space with min/max coordinates
#[derive(Debug, Clone, PartialEq)]
struct BoundingBox {
    xmin: f64,
    ymin: f64,
    xmax: f64,
    ymax: f64,
}

impl BoundingBox {
    fn new(xmin: f64, ymin: f64, xmax: f64, ymax: f64) -> Self {
        BoundingBox {
            xmin,
            ymin,
            xmax,
            ymax,
        }
    }
}

impl Point for BoundingBox {
    // Get value for dimension (0=xmin, 1=ymin, 2=xmax, 3=ymax)
    fn get_dimension(&self, dim: usize) -> f64 {
        match dim {
            0 => self.xmin,
            1 => self.ymin,
            2 => self.xmax,
            3 => self.ymax,
            _ => panic!("Invalid dimension: {}", dim),
        }
    }

    fn dimensions(&self) -> usize {
        4
    }
}

impl SpatialPoint for BoundingBox {
    // Check if this bounding box is fully within the query box
    fn is_within(&self, query: &Self) -> bool {
        self.xmin >= query.xmin
            && self.xmax <= query.xmax
            && self.ymin >= query.ymin
            && self.ymax <= query.ymax
    }

    // Check if this bounding box overlaps with the query box
    fn overlaps(&self, query: &Self) -> bool {
        !(self.xmax < query.xmin
            || self.xmin > query.xmax
            || self.ymax < query.ymin
            || self.ymin > query.ymax)
    }
}

// KD-tree node with generic point type and associated data
// Uses raw pointers for non-owning references (arena allocation pattern)
// P: Point type (coordinates, bounding box, etc.)
// T: Associated data type (page_id, user data, etc.)
struct Node<P: Point, T> {
    point: P,                       // Spatial data for tree algorithms
    data: T,                        // Associated payload (page_id, etc.)
    left: Option<*mut Node<P, T>>,  // Left child (non-owning pointer)
    right: Option<*mut Node<P, T>>, // Right child (non-owning pointer)
}

// Abstract interface for linking nodes in different storage strategies
// Allows same KD-tree algorithms to work with memory pointers, file offsets, or compressed blocks
trait NodeLinker<P: Point, T> {
    type NodeRef: Copy + Clone; // Reference to a node (pointer, offset, index, etc.)

    // Core linking operations
    fn link_left(&mut self, parent: Self::NodeRef, child: Self::NodeRef);
    fn link_right(&mut self, parent: Self::NodeRef, child: Self::NodeRef);

    // Navigation during traversal
    fn get_left(&self, node: Self::NodeRef) -> Option<Self::NodeRef>;
    fn get_right(&self, node: Self::NodeRef) -> Option<Self::NodeRef>;

    // Data access during algorithms
    fn get_point(&self, node: Self::NodeRef) -> &P;
    fn get_data(&self, node: Self::NodeRef) -> &T;
}

// In-memory implementation using raw pointers for arena allocation
// Simple and fast for development and small trees
struct InMemoryLinker;

impl<P: Point, T> NodeLinker<P, T> for InMemoryLinker {
    type NodeRef = *mut Node<P, T>;

    fn link_left(&mut self, parent: Self::NodeRef, child: Self::NodeRef) {
        unsafe {
            (*parent).left = Some(child);
        }
    }

    fn link_right(&mut self, parent: Self::NodeRef, child: Self::NodeRef) {
        unsafe {
            (*parent).right = Some(child);
        }
    }

    fn get_left(&self, node: Self::NodeRef) -> Option<Self::NodeRef> {
        unsafe { (*node).left }
    }

    fn get_right(&self, node: Self::NodeRef) -> Option<Self::NodeRef> {
        unsafe { (*node).right }
    }

    fn get_point(&self, node: Self::NodeRef) -> &P {
        unsafe { &(*node).point }
    }

    fn get_data(&self, node: Self::NodeRef) -> &T {
        unsafe { &(*node).data }
    }
}

// Simple KD-tree insertion function demonstrating "tree tools" approach
// Takes a linker and inserts a node into the tree using alternating dimensions
fn insert_node<P: Point, T, L: NodeLinker<P, T>>(
    linker: &mut L,
    root: Option<L::NodeRef>,
    new_node: L::NodeRef,
    depth: usize,
) -> L::NodeRef {
    // If no root exists, this becomes the root
    let Some(current_root) = root else {
        return new_node;
    };

    // Get the current dimension to split on (alternating by depth)
    let current_point = linker.get_point(current_root);
    let new_point = linker.get_point(new_node);
    let dimension = depth % new_point.dimensions();

    // Compare along the current dimension
    let new_coord = new_point.get_dimension(dimension);
    let current_coord = current_point.get_dimension(dimension);

    if new_coord < current_coord {
        // Go left
        if let Some(left_child) = linker.get_left(current_root) {
            insert_node(linker, Some(left_child), new_node, depth + 1);
        } else {
            linker.link_left(current_root, new_node);
        }
    } else {
        // Go right
        if let Some(right_child) = linker.get_right(current_root) {
            insert_node(linker, Some(right_child), new_node, depth + 1);
        } else {
            linker.link_right(current_root, new_node);
        }
    }

    current_root
}

// KD-tree structure following "tree tools" approach
// Contains arena for node allocation and root pointer
// Algorithms operate on this structure via separate functions
// TODO: Implement when we need arena allocation
// struct KDTree<P: Point, T> { ... }

fn main() {
    // Test KD-tree insertion using our "tree tools" approach

    // Create multiple bounding box nodes for testing
    let mut nodes = vec![
        Node {
            point: BoundingBox::new(5.0, 5.0, 6.0, 6.0), // Root
            data: 1u32,
            left: None,
            right: None,
        },
        Node {
            point: BoundingBox::new(2.0, 2.0, 3.0, 3.0), // Should go left (xmin < 5.0)
            data: 2u32,
            left: None,
            right: None,
        },
        Node {
            point: BoundingBox::new(8.0, 8.0, 9.0, 9.0), // Should go right (xmin >= 5.0)
            data: 3u32,
            left: None,
            right: None,
        },
    ];

    let mut linker = InMemoryLinker;

    // Get node references
    let root_ref = &mut nodes[0] as *mut Node<BoundingBox, u32>;
    let node2_ref = &mut nodes[1] as *mut Node<BoundingBox, u32>;
    let node3_ref = &mut nodes[2] as *mut Node<BoundingBox, u32>;

    // Build tree using our insertion algorithm
    let root = insert_node(&mut linker, None, root_ref, 0);
    insert_node(&mut linker, Some(root), node2_ref, 0);
    insert_node(&mut linker, Some(root), node3_ref, 0);

    // Verify tree structure
    println!("Root: data={}", linker.get_data(root));

    if let Some(left) = linker.get_left(root) {
        let left_point = linker.get_point(left);
        println!(
            "  Left child: xmin={}, data={}",
            left_point.xmin,
            linker.get_data(left)
        );
    }

    if let Some(right) = linker.get_right(root) {
        let right_point = linker.get_point(right);
        println!(
            "  Right child: xmin={}, data={}",
            right_point.xmin,
            linker.get_data(right)
        );
    }

    println!("KD-tree insertion working with NodeLinker abstraction!");
}
