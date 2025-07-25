/*
LUCENE-INSPIRED BKD TREE IMPLEMENTATION WITH GENERIC STORAGE ABSTRACTION

ARCHITECTURE OVERVIEW:
This implementation provides a generic, storage-agnostic KD-tree following Lucene's BKD design
principles but with Rust's type system for safety and performance.

KEY DESIGN PATTERNS:

1. EXTERNAL ARENA PATTERN
   - NodeArena handles allocation (user responsibility)
   - NodeLinker handles navigation (algorithm responsibility)
   - Clean separation: allocation != linking != tree algorithms

2. STORAGE ABSTRACTION
   - NodeLinker trait: same algorithms work with any storage backend
   - Current: InMemoryLinker with arena indices
   - Future: TantivyLinker with file offsets, CompressedLinker with blocks

3. DIMENSIONAL ALTERNATION
   - Tree depth determines split dimension: depth % point.dimensions()
   - 4D bounding boxes: [xmin, ymin, xmax, ymax] cycle through 0,1,2,3
   - Matches Lucene BKD behavior for consistent spatial partitioning

4. SPATIAL SEARCH WITH PRUNING
   - Based on bbox.rs implementation but made generic
   - Dimensional pruning: only visit subtrees that could contain results
   - Handles both fully-within and overlapping spatial relationships

INSPIRATIONS:
- Lucene BKD: Spatial indexing, dimensional alternation, block-based storage
- Tantivy: Rust implementation patterns, memory-mapped file handling
- bbox.rs: Concrete spatial search behavior to generalize

FUTURE EVOLUTION:
- Tantivy integration: file-based storage with memory mapping
- Block compression: 1024-node blocks like Lucene BKD
- Multiple point types: 2D points, 3D points, geographic coordinates
- Different data payloads: page_ids, document_ids, custom metadata
*/

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
// Uses indices for arena allocation pattern
// P: Point type (coordinates, bounding box, etc.)
// T: Associated data type (page_id, user data, etc.)
struct Node<P: Point, T> {
    point: P,             // Spatial data for tree algorithms
    data: T,              // Associated payload (page_id, etc.)
    left: Option<usize>,  // Left child (arena index)
    right: Option<usize>, // Right child (arena index)
}

impl<P: Point, T> Node<P, T> {
    fn get_point(&self) -> &P {
        &self.point
    }

    fn get_data(&self) -> &T {
        &self.data
    }
}

// CORE ABSTRACTION: NodeLinker trait enables storage-agnostic KD-tree algorithms
//
// DESIGN PRINCIPLE: Separation of concerns between allocation and linking
// - NodeArena handles allocation (external to linker - user responsibility)
// - NodeLinker handles navigation and data access (this trait)
// - Tree algorithms use linker interface, work with any storage backend
//
// FUTURE BACKENDS: Same algorithms will work with:
// - InMemoryLinker (current): arena indices
// - TantivyLinker: file offsets, memory-mapped pages
// - CompressedLinker: block-based storage with decompression
trait NodeLinker<P: Point, T> {
    type NodeRef: Copy + Clone; // Reference to a node (pointer, offset, index, etc.)

    // Core linking operations - modify tree structure
    fn link_left(&mut self, parent: Self::NodeRef, child: Self::NodeRef);
    fn link_right(&mut self, parent: Self::NodeRef, child: Self::NodeRef);

    // Navigation during traversal - read-only operations
    fn get_left(&self, node: Self::NodeRef) -> Option<Self::NodeRef>;
    fn get_right(&self, node: Self::NodeRef) -> Option<Self::NodeRef>;

    // Data access during algorithms - read-only operations
    fn get_point(&self, node: Self::NodeRef) -> &P;
    fn get_data(&self, node: Self::NodeRef) -> &T;
}

// Arena-based allocator for in-memory nodes
// Manages node allocation and provides stable references
struct NodeArena<P: Point, T> {
    nodes: Vec<Node<P, T>>,
}

impl<P: Point, T> NodeArena<P, T> {
    fn new() -> Self {
        NodeArena { nodes: Vec::new() }
    }

    fn with_capacity(capacity: usize) -> Self {
        NodeArena {
            nodes: Vec::with_capacity(capacity),
        }
    }

    // Allocate a new node and return its index
    fn allocate(&mut self, point: P, data: T) -> usize {
        let index = self.nodes.len();
        self.nodes.push(Node {
            point,
            data,
            left: None,
            right: None,
        });
        index
    }

    // Get a reference to a node by index
    fn get(&self, index: usize) -> &Node<P, T> {
        &self.nodes[index]
    }

    // Get a mutable reference to a node by index
    fn get_mut(&mut self, index: usize) -> &mut Node<P, T> {
        &mut self.nodes[index]
    }

    fn len(&self) -> usize {
        self.nodes.len()
    }
}

// EXTERNAL ARENA PATTERN: InMemoryLinker takes arena reference, doesn't own allocation
//
// ARCHITECTURE DECISION: User controls node allocation, linker only handles linking
// Benefits:
// - Separation of concerns: allocation vs linking are independent
// - Flexibility: user can pre-allocate, batch allocate, or use custom allocation strategies
// - Reusability: same linker can work with different arena instances
// - Testing: easier to create controlled test scenarios
//
// Usage pattern:
//   let mut arena = NodeArena::new();
//   let node1 = arena.allocate(point, data);  // User allocates
//   let mut linker = InMemoryLinker::new(&mut arena);  // Linker borrows arena
//   // Tree algorithms use linker, don't see arena directly
struct InMemoryLinker<'a, P: Point, T> {
    arena: &'a mut NodeArena<P, T>,
}

impl<'a, P: Point, T> InMemoryLinker<'a, P, T> {
    fn new(arena: &'a mut NodeArena<P, T>) -> Self {
        InMemoryLinker { arena }
    }
}

impl<'a, P: Point, T> NodeLinker<P, T> for InMemoryLinker<'a, P, T> {
    type NodeRef = usize; // Use index instead of raw pointer

    fn link_left(&mut self, parent: Self::NodeRef, child: Self::NodeRef) {
        self.arena.get_mut(parent).left = Some(child);
    }

    fn link_right(&mut self, parent: Self::NodeRef, child: Self::NodeRef) {
        self.arena.get_mut(parent).right = Some(child);
    }

    fn get_left(&self, node: Self::NodeRef) -> Option<Self::NodeRef> {
        self.arena.get(node).left
    }

    fn get_right(&self, node: Self::NodeRef) -> Option<Self::NodeRef> {
        self.arena.get(node).right
    }

    fn get_point(&self, node: Self::NodeRef) -> &P {
        self.arena.get(node).get_point()
    }

    fn get_data(&self, node: Self::NodeRef) -> &T {
        self.arena.get(node).get_data()
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

// Generic spatial search function for KD-tree using NodeLinker abstraction
// Returns all nodes whose spatial data overlaps with or is within the query
//
// ARCHITECTURE: This implements the same spatial pruning logic as bbox.rs but generically:
// - Uses NodeLinker abstraction to work with any storage backend (memory, files, compressed)
// - Employs dimensional pruning: only visits subtrees that could contain overlapping results
// - Alternates dimensions by depth: root splits on dim 0, children on dim 1, etc.
// - For 4D bounding boxes: [xmin, ymin, xmax, ymax] cycle through dimensions 0,1,2,3
fn spatial_search<P: SpatialPoint, T, L: NodeLinker<P, T>>(
    linker: &L,
    root: Option<L::NodeRef>,
    query: &P,
    depth: usize,
) -> Vec<L::NodeRef> {
    let mut results = Vec::new();

    if let Some(current_node) = root {
        spatial_search_recursive(linker, current_node, query, depth, &mut results);
    }

    results
}

fn spatial_search_recursive<P: SpatialPoint, T, L: NodeLinker<P, T>>(
    linker: &L,
    node: L::NodeRef,
    query: &P,
    depth: usize,
    results: &mut Vec<L::NodeRef>,
) {
    let node_point = linker.get_point(node);

    // Check if this node should be included in results
    // BEHAVIOR: Matches bbox.rs - collect nodes that are fully within OR partially overlap query
    if node_point.is_within(query) || node_point.overlaps(query) {
        results.push(node);
    }

    // DIMENSIONAL PRUNING: Determine which children to visit based on current dimension split
    // This is the core optimization - only visit subtrees that could contain overlapping results
    let dimension = depth % query.dimensions();
    let split_value = node_point.get_dimension(dimension);

    // Get query bounds for this dimension - handles 4D bounding box logic
    let query_min = query.get_dimension(dimension);
    let query_max = if dimension < 2 {
        // For min dimensions (xmin=0, ymin=1), check if query extends into this subspace
        // Query [1,2,5,6] on xmin split: need to check if query.xmax >= split_value
        query.get_dimension(dimension + 2) // xmax or ymax
    } else {
        // For max dimensions (xmax=2, ymax=3), query bound is the same coordinate
        // Query [1,2,5,6] on xmax split: check if query.xmax >= split_value
        query_min
    };

    // PRUNING LOGIC: Only recurse if query could overlap that subspace
    // Left subtree: contains values <= split_value
    if let Some(left_child) = linker.get_left(node) {
        if query_min <= split_value {
            spatial_search_recursive(linker, left_child, query, depth + 1, results);
        }
    }

    // Right subtree: contains values >= split_value
    if let Some(right_child) = linker.get_right(node) {
        if query_max >= split_value {
            spatial_search_recursive(linker, right_child, query, depth + 1, results);
        }
    }
}

// KD-tree structure following "tree tools" approach
// Contains arena for node allocation and root pointer
// Algorithms operate on this structure via separate functions
// TODO: Implement when we need arena allocation
// struct KDTree<P: Point, T> { ... }

fn main() {
    println!("KD-tree implementation with NodeLinker abstraction and spatial search");
    println!("Run `cargo test` to execute the test suite");

    // Demonstrate spatial search functionality
    println!("\n=== Spatial Search Demo ===");

    let mut arena = NodeArena::new();

    // Create some bounding boxes representing different regions
    let store_ref = arena.allocate(BoundingBox::new(5.0, 5.0, 7.0, 7.0), 101); // Store location
    let house_ref = arena.allocate(BoundingBox::new(2.0, 2.0, 3.0, 3.0), 102); // House location
    let park_ref = arena.allocate(BoundingBox::new(8.0, 1.0, 10.0, 2.0), 103); // Park location
    let school_ref = arena.allocate(BoundingBox::new(1.0, 8.0, 2.0, 9.0), 104); // School location

    let mut linker = InMemoryLinker::new(&mut arena);

    // Build the spatial index
    let root = insert_node(&mut linker, None, store_ref, 0);
    insert_node(&mut linker, Some(root), house_ref, 0);
    insert_node(&mut linker, Some(root), park_ref, 0);
    insert_node(&mut linker, Some(root), school_ref, 0);

    // Search for locations within downtown area [0, 0, 6, 6]
    let downtown_query = BoundingBox::new(0.0, 0.0, 6.0, 6.0);
    let downtown_results = spatial_search(&linker, Some(root), &downtown_query, 0);

    println!("Searching downtown area [0,0 to 6,6]:");
    for &result_ref in &downtown_results {
        let point = linker.get_point(result_ref);
        let data = linker.get_data(result_ref);
        println!(
            "  Found location ID {}: [{}, {}, {}, {}]",
            data, point.xmin, point.ymin, point.xmax, point.ymax
        );
    }

    // Search for locations in a different area
    let eastside_query = BoundingBox::new(7.0, 0.0, 12.0, 5.0);
    let eastside_results = spatial_search(&linker, Some(root), &eastside_query, 0);

    println!("\nSearching eastside area [7,0 to 12,5]:");
    for &result_ref in &eastside_results {
        let point = linker.get_point(result_ref);
        let data = linker.get_data(result_ref);
        println!(
            "  Found location ID {}: [{}, {}, {}, {}]",
            data, point.xmin, point.ymin, point.xmax, point.ymax
        );
    }
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
        let mut arena = NodeArena::new();
        let root_ref = arena.allocate(BoundingBox::new(5.0, 5.0, 6.0, 6.0), 1);
        let mut linker = InMemoryLinker::new(&mut arena);

        let root = insert_node(&mut linker, None, root_ref, 0);

        // Root should be the node itself
        assert_eq!(linker.get_data(root), &1);
        assert_eq!(linker.get_point(root).xmin, 5.0);
        assert!(linker.get_left(root).is_none());
        assert!(linker.get_right(root).is_none());
        assert_eq!(arena.len(), 1);
    }

    #[test]
    fn test_basic_tree_construction() {
        let mut arena = NodeArena::with_capacity(3);

        // Allocate nodes using the arena
        let root_ref = arena.allocate(BoundingBox::new(5.0, 5.0, 6.0, 6.0), 1);
        let node2_ref = arena.allocate(BoundingBox::new(2.0, 2.0, 3.0, 3.0), 2); // Should go left (xmin < 5.0)
        let node3_ref = arena.allocate(BoundingBox::new(8.0, 8.0, 9.0, 9.0), 3); // Should go right (xmin >= 5.0)

        let mut linker = InMemoryLinker::new(&mut arena);

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

        assert_eq!(arena.len(), 3);
    }

    #[test]
    fn test_dimensional_alternation() {
        let mut arena = NodeArena::new();

        // Allocate nodes that will test different dimensions
        let root_ref = arena.allocate(BoundingBox::new(5.0, 5.0, 6.0, 6.0), 1); // Root (splits on dim 0: xmin)
        let node2_ref = arena.allocate(BoundingBox::new(2.0, 8.0, 3.0, 9.0), 2); // Left child (splits on dim 1: ymin)
        let node3_ref = arena.allocate(BoundingBox::new(1.0, 2.0, 1.5, 2.5), 3); // Should go left-left (ymin < 8.0)

        let mut linker = InMemoryLinker::new(&mut arena);

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
        let mut arena = NodeArena::with_capacity(5);

        // Allocate nodes for a more complex tree with 5 nodes
        let root_ref = arena.allocate(BoundingBox::new(5.0, 5.0, 6.0, 6.0), 1); // Root
        let left_ref = arena.allocate(BoundingBox::new(2.0, 2.0, 3.0, 3.0), 2); // Left
        let right_ref = arena.allocate(BoundingBox::new(8.0, 8.0, 9.0, 9.0), 3); // Right
        let left_left_ref = arena.allocate(BoundingBox::new(1.0, 1.0, 1.5, 1.5), 4); // Left-Left
        let right_right_ref = arena.allocate(BoundingBox::new(9.0, 9.0, 9.5, 9.5), 5); // Right-Right

        let mut linker = InMemoryLinker::new(&mut arena);

        // Build tree
        let root = insert_node(&mut linker, None, root_ref, 0);
        insert_node(&mut linker, Some(root), left_ref, 0);
        insert_node(&mut linker, Some(root), right_ref, 0);
        insert_node(&mut linker, Some(root), left_left_ref, 0);
        insert_node(&mut linker, Some(root), right_right_ref, 0);

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

        assert_eq!(arena.len(), 5);
    }

    #[test]
    fn test_inmemory_linker_operations() {
        let mut arena = NodeArena::new();

        // Allocate nodes using the arena
        let parent_ref = arena.allocate(BoundingBox::new(1.0, 1.0, 2.0, 2.0), 1);
        let child_ref = arena.allocate(BoundingBox::new(3.0, 3.0, 4.0, 4.0), 2);

        let mut linker = InMemoryLinker::new(&mut arena);

        // Test linking operations
        linker.link_left(parent_ref, child_ref);

        let linked_left = linker.get_left(parent_ref).unwrap();
        assert_eq!(linker.get_data(linked_left), &2);
        assert_eq!(linker.get_point(linked_left).xmin, 3.0);

        // Test that right is still None
        assert!(linker.get_right(parent_ref).is_none());

        assert_eq!(arena.len(), 2);
    }

    #[test]
    fn test_node_methods() {
        let node = create_test_node(1.0, 2.0, 3.0, 4.0, 42);

        // Test the new Node methods
        assert_eq!(node.get_data(), &42);
        let point = node.get_point();
        assert_eq!(point.xmin, 1.0);
        assert_eq!(point.ymin, 2.0);
        assert_eq!(point.xmax, 3.0);
        assert_eq!(point.ymax, 4.0);
    }

    #[test]
    fn test_spatial_search_single_match() {
        let mut arena = NodeArena::new();

        // Create nodes with different bounding boxes
        let root_ref = arena.allocate(BoundingBox::new(5.0, 5.0, 6.0, 6.0), 1); // Root
        let left_ref = arena.allocate(BoundingBox::new(2.0, 2.0, 3.0, 3.0), 2); // Left child
        let right_ref = arena.allocate(BoundingBox::new(8.0, 8.0, 9.0, 9.0), 3); // Right child

        let mut linker = InMemoryLinker::new(&mut arena);

        // Build tree
        let root = insert_node(&mut linker, None, root_ref, 0);
        insert_node(&mut linker, Some(root), left_ref, 0);
        insert_node(&mut linker, Some(root), right_ref, 0);

        // Query for bounding boxes that overlap with [1.0, 1.0, 4.0, 4.0]
        let query = BoundingBox::new(1.0, 1.0, 4.0, 4.0);
        let results = spatial_search(&linker, Some(root), &query, 0);

        // Should find root (overlaps) and left child (overlaps)
        // Right child should not be found (no overlap with [8.0, 8.0, 9.0, 9.0])
        assert!(results.len() >= 1); // At least left child

        // Verify the results contain expected data
        let result_data: Vec<u32> = results
            .iter()
            .map(|&node_ref| *linker.get_data(node_ref))
            .collect();

        // Left child (data=2) should definitely be in results
        assert!(result_data.contains(&2));
    }

    #[test]
    fn test_spatial_search_no_matches() {
        let mut arena = NodeArena::new();

        // Create nodes in one area
        let root_ref = arena.allocate(BoundingBox::new(10.0, 10.0, 11.0, 11.0), 1);
        let left_ref = arena.allocate(BoundingBox::new(12.0, 12.0, 13.0, 13.0), 2);

        let mut linker = InMemoryLinker::new(&mut arena);

        // Build tree
        let root = insert_node(&mut linker, None, root_ref, 0);
        insert_node(&mut linker, Some(root), left_ref, 0);

        // Query in a completely different area
        let query = BoundingBox::new(1.0, 1.0, 2.0, 2.0);
        let results = spatial_search(&linker, Some(root), &query, 0);

        // Should find no matches
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_spatial_search_within_query() {
        let mut arena = NodeArena::new();

        // Create small bounding boxes inside a larger query area
        let root_ref = arena.allocate(BoundingBox::new(3.0, 3.0, 4.0, 4.0), 1); // Inside query
        let left_ref = arena.allocate(BoundingBox::new(2.0, 2.0, 2.5, 2.5), 2); // Inside query
        let right_ref = arena.allocate(BoundingBox::new(15.0, 15.0, 16.0, 16.0), 3); // Outside query

        let mut linker = InMemoryLinker::new(&mut arena);

        // Build tree
        let root = insert_node(&mut linker, None, root_ref, 0);
        insert_node(&mut linker, Some(root), left_ref, 0);
        insert_node(&mut linker, Some(root), right_ref, 0);

        // Large query box that contains root and left child
        let query = BoundingBox::new(1.0, 1.0, 5.0, 5.0);
        let results = spatial_search(&linker, Some(root), &query, 0);

        let result_data: Vec<u32> = results
            .iter()
            .map(|&node_ref| *linker.get_data(node_ref))
            .collect();

        // Should find root (data=1) and left child (data=2)
        assert!(result_data.contains(&1));
        assert!(result_data.contains(&2));
        // Should not find right child (data=3) - it's outside query area
        assert!(!result_data.contains(&3));
    }

    #[test]
    fn test_spatial_search_dimensional_pruning() {
        let mut arena = NodeArena::with_capacity(7);

        // Build a tree that tests dimensional pruning across multiple levels
        // Root splits on xmin (dimension 0)
        let root_ref = arena.allocate(BoundingBox::new(10.0, 10.0, 11.0, 11.0), 1);

        // Left subtree (xmin < 10.0) - splits on ymin (dimension 1)
        let left_ref = arena.allocate(BoundingBox::new(5.0, 15.0, 6.0, 16.0), 2);
        let left_left_ref = arena.allocate(BoundingBox::new(3.0, 8.0, 4.0, 9.0), 3); // ymin < 15.0
        let left_right_ref = arena.allocate(BoundingBox::new(7.0, 20.0, 8.0, 21.0), 4); // ymin >= 15.0

        // Right subtree (xmin >= 10.0) - splits on ymin (dimension 1)
        let right_ref = arena.allocate(BoundingBox::new(15.0, 5.0, 16.0, 6.0), 5);
        let right_left_ref = arena.allocate(BoundingBox::new(20.0, 2.0, 21.0, 3.0), 6); // ymin < 5.0
        let right_right_ref = arena.allocate(BoundingBox::new(25.0, 8.0, 26.0, 9.0), 7); // ymin >= 5.0

        let mut linker = InMemoryLinker::new(&mut arena);

        // Build tree
        let root = insert_node(&mut linker, None, root_ref, 0);
        insert_node(&mut linker, Some(root), left_ref, 0);
        insert_node(&mut linker, Some(root), right_ref, 0);
        insert_node(&mut linker, Some(root), left_left_ref, 0);
        insert_node(&mut linker, Some(root), left_right_ref, 0);
        insert_node(&mut linker, Some(root), right_left_ref, 0);
        insert_node(&mut linker, Some(root), right_right_ref, 0);

        // Query that should only find nodes in left subtree with low y values
        // This tests that we properly prune the right subtree (xmin >= 10.0)
        // and also prune left_right (ymin >= 15.0)
        let query = BoundingBox::new(1.0, 1.0, 12.0, 12.0);
        let results = spatial_search(&linker, Some(root), &query, 0);

        let result_data: Vec<u32> = results
            .iter()
            .map(|&node_ref| *linker.get_data(node_ref))
            .collect();

        // Should find:
        // - root (data=1): overlaps with query
        // - left_left (data=3): overlaps with query [3.0,8.0,4.0,9.0] vs [1.0,1.0,12.0,12.0]
        assert!(result_data.contains(&1), "Root should be found");
        assert!(result_data.contains(&3), "Left-left should be found");

        // Should NOT find (due to dimensional pruning):
        // - left (data=2): [5.0,15.0,6.0,16.0] doesn't overlap [1.0,1.0,12.0,12.0]
        // - left_right (data=4): [7.0,20.0,8.0,21.0] doesn't overlap query
        // - right (data=5): [15.0,5.0,16.0,6.0] doesn't overlap query (xmin too high)
        // - right_left (data=6): in right subtree, should be pruned
        // - right_right (data=7): in right subtree, should be pruned
        assert!(
            !result_data.contains(&2),
            "Left should not be found (no overlap)"
        );
        assert!(
            !result_data.contains(&4),
            "Left-right should not be found (no overlap)"
        );
        assert!(
            !result_data.contains(&5),
            "Right should not be found (pruned by xmin)"
        );
        assert!(
            !result_data.contains(&6),
            "Right-left should not be found (pruned)"
        );
        assert!(
            !result_data.contains(&7),
            "Right-right should not be found (pruned)"
        );

        println!("Spatial search found nodes with data: {:?}", result_data);
    }

    #[test]
    fn test_spatial_search_within_vs_overlapping() {
        let mut arena = NodeArena::new();

        // Create nodes with different spatial relationships to query
        let fully_within_ref = arena.allocate(BoundingBox::new(2.0, 2.0, 3.0, 3.0), 1); // Fully within
        let overlapping_ref = arena.allocate(BoundingBox::new(4.5, 4.5, 6.0, 6.0), 2); // Overlaps
        let outside_ref = arena.allocate(BoundingBox::new(8.0, 8.0, 9.0, 9.0), 3); // Outside

        let mut linker = InMemoryLinker::new(&mut arena);

        // Build simple tree with overlapping as root
        let root = insert_node(&mut linker, None, overlapping_ref, 0);
        insert_node(&mut linker, Some(root), fully_within_ref, 0);
        insert_node(&mut linker, Some(root), outside_ref, 0);

        // Query box that contains fully_within and overlaps with overlapping
        let query = BoundingBox::new(1.0, 1.0, 5.0, 5.0);
        let results = spatial_search(&linker, Some(root), &query, 0);

        let result_data: Vec<u32> = results
            .iter()
            .map(|&node_ref| *linker.get_data(node_ref))
            .collect();

        // Verify the spatial relationships
        let fully_within_bbox = BoundingBox::new(2.0, 2.0, 3.0, 3.0);
        let overlapping_bbox = BoundingBox::new(4.5, 4.5, 6.0, 6.0);
        let outside_bbox = BoundingBox::new(8.0, 8.0, 9.0, 9.0);

        assert!(
            fully_within_bbox.is_within(&query),
            "fully_within should be within query"
        );
        assert!(
            overlapping_bbox.overlaps(&query),
            "overlapping should overlap query"
        );
        assert!(
            !overlapping_bbox.is_within(&query),
            "overlapping should not be fully within query"
        );
        assert!(
            !outside_bbox.overlaps(&query),
            "outside should not overlap query"
        );

        // Results should include both fully_within and overlapping
        assert!(
            result_data.contains(&1),
            "Fully within node should be found"
        );
        assert!(result_data.contains(&2), "Overlapping node should be found");
        assert!(
            !result_data.contains(&3),
            "Outside node should not be found"
        );

        println!("Query box: {:?}", query);
        println!(
            "Fully within: {:?} -> found: {}",
            fully_within_bbox,
            result_data.contains(&1)
        );
        println!(
            "Overlapping: {:?} -> found: {}",
            overlapping_bbox,
            result_data.contains(&2)
        );
        println!(
            "Outside: {:?} -> found: {}",
            outside_bbox,
            result_data.contains(&3)
        );
    }

    #[test]
    fn test_tantivy_components_availability() {
        #[cfg(test)]
        {
            // Test that we can access Tantivy's key components for future TantivyLinker implementation
            use tantivy::directory::{MmapDirectory, RamDirectory};

            // Test memory-mapped directory (for file-based node storage)
            let _mmap_directory = MmapDirectory::create_from_tempdir().unwrap();

            // Test in-memory directory (for testing)
            let _ram_directory = RamDirectory::create();

            // This test verifies we can access the components we identified as general-purpose:
            // - MmapDirectory: File-based storage with memory mapping
            // - RamDirectory: In-memory storage for testing
            // Future: We'll also test OwnedBytes and compression components
            println!("Tantivy components test: Memory mapping and RAM directories available");
        }
    }

    #[test]
    fn test_tantivy_general_purpose_components() {
        #[cfg(test)]
        {
            // Test the specific general-purpose components we identified:
            // 1. OwnedBytes for memory management
            // 2. Compression algorithms (when available with feature flags)

            use tantivy::directory::OwnedBytes;

            // Test OwnedBytes - general-purpose memory management
            let test_data = vec![1u8, 2, 3, 4, 5];
            let owned_bytes = OwnedBytes::new(test_data);
            assert_eq!(owned_bytes.len(), 5);
            assert_eq!(owned_bytes.as_slice(), &[1, 2, 3, 4, 5]);

            // Test slicing - important for block-based node storage
            let slice = owned_bytes.slice(1..4);
            assert_eq!(slice.as_slice(), &[2, 3, 4]);

            println!("Tantivy OwnedBytes test: Memory management primitives working");

            // Future tests will add:
            // - Compression/decompression with LZ4 and Zstd
            // - File I/O operations with MmapDirectory
            // - Integration with our NodeLinker trait
        }
    }
}
