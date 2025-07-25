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
    println!("KD-tree implementation with NodeLinker abstraction");
    println!("Run `cargo test` to execute the test suite");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a test node
    fn create_test_node(
        xmin: f64,
        ymin: f64,
        xmax: f64,
        ymax: f64,
        data: u32,
    ) -> Node<BoundingBox, u32> {
        Node {
            point: BoundingBox::new(xmin, ymin, xmax, ymax),
            data,
            left: None,
            right: None,
        }
    }

    #[test]
    fn test_bounding_box_point_trait() {
        let bbox = BoundingBox::new(1.0, 2.0, 3.0, 4.0);

        assert_eq!(bbox.get_dimension(0), 1.0); // xmin
        assert_eq!(bbox.get_dimension(1), 2.0); // ymin
        assert_eq!(bbox.get_dimension(2), 3.0); // xmax
        assert_eq!(bbox.get_dimension(3), 4.0); // ymax
        assert_eq!(bbox.dimensions(), 4);
    }

    #[test]
    fn test_bounding_box_spatial_operations() {
        let bbox1 = BoundingBox::new(1.0, 1.0, 3.0, 3.0);
        let bbox2 = BoundingBox::new(2.0, 2.0, 4.0, 4.0); // Overlaps bbox1
        let bbox3 = BoundingBox::new(0.0, 0.0, 5.0, 5.0); // Contains bbox1
        let bbox4 = BoundingBox::new(10.0, 10.0, 12.0, 12.0); // No overlap

        // Test overlaps
        assert!(bbox1.overlaps(&bbox2));
        assert!(bbox1.overlaps(&bbox3));
        assert!(!bbox1.overlaps(&bbox4));

        // Test is_within
        assert!(bbox1.is_within(&bbox3));
        assert!(!bbox1.is_within(&bbox2));
        assert!(!bbox1.is_within(&bbox4));
    }

    #[test]
    fn test_single_node_insertion() {
        let mut nodes = vec![create_test_node(5.0, 5.0, 6.0, 6.0, 1)];
        let mut linker = InMemoryLinker;
        let root_ref = &mut nodes[0] as *mut Node<BoundingBox, u32>;

        let root = insert_node(&mut linker, None, root_ref, 0);

        // Root should be the node itself
        assert_eq!(linker.get_data(root), &1);
        assert_eq!(linker.get_point(root).xmin, 5.0);
        assert!(linker.get_left(root).is_none());
        assert!(linker.get_right(root).is_none());
    }

    #[test]
    fn test_basic_tree_construction() {
        // Create test nodes - same as original main function test
        let mut nodes = vec![
            create_test_node(5.0, 5.0, 6.0, 6.0, 1), // Root
            create_test_node(2.0, 2.0, 3.0, 3.0, 2), // Should go left (xmin < 5.0)
            create_test_node(8.0, 8.0, 9.0, 9.0, 3), // Should go right (xmin >= 5.0)
        ];

        let mut linker = InMemoryLinker;

        // Get node references
        let root_ref = &mut nodes[0] as *mut Node<BoundingBox, u32>;
        let node2_ref = &mut nodes[1] as *mut Node<BoundingBox, u32>;
        let node3_ref = &mut nodes[2] as *mut Node<BoundingBox, u32>;

        // Build tree
        let root = insert_node(&mut linker, None, root_ref, 0);
        insert_node(&mut linker, Some(root), node2_ref, 0);
        insert_node(&mut linker, Some(root), node3_ref, 0);

        // Verify root
        assert_eq!(linker.get_data(root), &1);
        assert_eq!(linker.get_point(root).xmin, 5.0);

        // Verify left child (should be node with data=2, xmin=2.0)
        let left = linker.get_left(root).expect("Root should have left child");
        assert_eq!(linker.get_data(left), &2);
        assert_eq!(linker.get_point(left).xmin, 2.0);

        // Verify right child (should be node with data=3, xmin=8.0)
        let right = linker
            .get_right(root)
            .expect("Root should have right child");
        assert_eq!(linker.get_data(right), &3);
        assert_eq!(linker.get_point(right).xmin, 8.0);
    }

    #[test]
    fn test_dimensional_alternation() {
        // Create nodes that will test different dimensions
        let mut nodes = vec![
            create_test_node(5.0, 5.0, 6.0, 6.0, 1), // Root (splits on dim 0: xmin)
            create_test_node(2.0, 8.0, 3.0, 9.0, 2), // Left child (splits on dim 1: ymin)
            create_test_node(1.0, 2.0, 1.5, 2.5, 3), // Should go left-left (ymin < 8.0)
        ];

        let mut linker = InMemoryLinker;

        let root_ref = &mut nodes[0] as *mut Node<BoundingBox, u32>;
        let node2_ref = &mut nodes[1] as *mut Node<BoundingBox, u32>;
        let node3_ref = &mut nodes[2] as *mut Node<BoundingBox, u32>;

        // Build tree
        let root = insert_node(&mut linker, None, root_ref, 0);
        insert_node(&mut linker, Some(root), node2_ref, 0);
        insert_node(&mut linker, Some(root), node3_ref, 0);

        // Navigate to left child (data=2)
        let left = linker.get_left(root).expect("Root should have left child");
        assert_eq!(linker.get_data(left), &2);

        // Navigate to left-left child (data=3) - split on ymin dimension
        let left_left = linker
            .get_left(left)
            .expect("Left child should have left child");
        assert_eq!(linker.get_data(left_left), &3);
        assert_eq!(linker.get_point(left_left).ymin, 2.0);
    }

    #[test]
    fn test_deeper_tree_construction() {
        // Create a more complex tree with 5 nodes
        let mut nodes = vec![
            create_test_node(5.0, 5.0, 6.0, 6.0, 1), // Root
            create_test_node(2.0, 2.0, 3.0, 3.0, 2), // Left
            create_test_node(8.0, 8.0, 9.0, 9.0, 3), // Right
            create_test_node(1.0, 1.0, 1.5, 1.5, 4), // Left-Left
            create_test_node(9.0, 9.0, 9.5, 9.5, 5), // Right-Right
        ];

        let mut linker = InMemoryLinker;

        // Build tree
        let mut node_refs = Vec::new();
        for node in &mut nodes {
            node_refs.push(node as *mut Node<BoundingBox, u32>);
        }

        let root = insert_node(&mut linker, None, node_refs[0], 0);
        for &node_ref in &node_refs[1..] {
            insert_node(&mut linker, Some(root), node_ref, 0);
        }

        // Verify tree structure
        assert_eq!(linker.get_data(root), &1);

        let left = linker.get_left(root).unwrap();
        assert_eq!(linker.get_data(left), &2);

        let right = linker.get_right(root).unwrap();
        assert_eq!(linker.get_data(right), &3);

        let left_left = linker.get_left(left).unwrap();
        assert_eq!(linker.get_data(left_left), &4);

        let right_right = linker.get_right(right).unwrap();
        assert_eq!(linker.get_data(right_right), &5);
    }

    #[test]
    fn test_inmemory_linker_operations() {
        let mut nodes = vec![
            create_test_node(1.0, 1.0, 2.0, 2.0, 1),
            create_test_node(3.0, 3.0, 4.0, 4.0, 2),
        ];

        let mut linker = InMemoryLinker;
        let parent_ref = &mut nodes[0] as *mut Node<BoundingBox, u32>;
        let child_ref = &mut nodes[1] as *mut Node<BoundingBox, u32>;

        // Test linking operations
        linker.link_left(parent_ref, child_ref);

        let linked_left = linker.get_left(parent_ref).unwrap();
        assert_eq!(linker.get_data(linked_left), &2);
        assert_eq!(linker.get_point(linked_left).xmin, 3.0);

        // Test that right is still None
        assert!(linker.get_right(parent_ref).is_none());
    }
}
