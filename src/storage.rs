//! Storage abstractions and memory management for spatial indexes.

use crate::spatial::Point;

/// KD-tree node with generic point type and associated data.
/// Uses indices for arena allocation pattern.
/// P: Point type (coordinates, bounding box, etc.)
/// T: Associated data type (page_id, user data, etc.)
pub struct Node<P: Point, T> {
    pub point: P,             // Spatial data for tree algorithms
    pub data: T,              // Associated payload (page_id, etc.)
    pub left: Option<usize>,  // Left child (arena index)
    pub right: Option<usize>, // Right child (arena index)
}

impl<P: Point, T> Node<P, T> {
    /// Get a reference to the spatial point data.
    pub fn get_point(&self) -> &P {
        &self.point
    }

    /// Get a reference to the associated data.
    pub fn get_data(&self) -> &T {
        &self.data
    }
}

/// Core abstraction: NodeLinker trait enables storage-agnostic KD-tree algorithms.
///
/// # Design Principle: Separation of concerns between allocation and linking
/// - NodeArena handles allocation (external to linker - user responsibility)
/// - NodeLinker handles navigation and data access (this trait)
/// - Tree algorithms use linker interface, work with any storage backend
///
/// # Future Backends: Same algorithms will work with:
/// - InMemoryLinker (current): arena indices
/// - TantivyLinker: file offsets, memory-mapped pages
/// - CompressedLinker: block-based storage with decompression
pub trait NodeLinker<P: Point, T> {
    /// Reference to a node (pointer, offset, index, etc.)
    type NodeRef: Copy + Clone;

    // Core linking operations - modify tree structure
    /// Link a child as the left child of a parent node.
    fn link_left(&mut self, parent: Self::NodeRef, child: Self::NodeRef);

    /// Link a child as the right child of a parent node.
    fn link_right(&mut self, parent: Self::NodeRef, child: Self::NodeRef);

    // Navigation during traversal - read-only operations
    /// Get the left child of a node, if it exists.
    fn get_left(&self, node: Self::NodeRef) -> Option<Self::NodeRef>;

    /// Get the right child of a node, if it exists.
    fn get_right(&self, node: Self::NodeRef) -> Option<Self::NodeRef>;

    // Data access during algorithms - read-only operations
    /// Get a reference to the spatial point data of a node.
    fn get_point(&self, node: Self::NodeRef) -> &P;

    /// Get a reference to the associated data of a node.
    fn get_data(&self, node: Self::NodeRef) -> &T;
}

/// Arena-based allocator for in-memory nodes.
/// Manages node allocation and provides stable references.
pub struct NodeArena<P: Point, T> {
    nodes: Vec<Node<P, T>>,
}

impl<P: Point, T> NodeArena<P, T> {
    /// Create a new empty arena.
    pub fn new() -> Self {
        NodeArena { nodes: Vec::new() }
    }

    /// Create a new arena with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        NodeArena {
            nodes: Vec::with_capacity(capacity),
        }
    }

    /// Allocate a new node and return its index.
    pub fn allocate(&mut self, point: P, data: T) -> usize {
        let index = self.nodes.len();
        self.nodes.push(Node {
            point,
            data,
            left: None,
            right: None,
        });
        index
    }

    /// Get a reference to a node by index.
    pub fn get(&self, index: usize) -> &Node<P, T> {
        &self.nodes[index]
    }

    /// Get a mutable reference to a node by index.
    pub fn get_mut(&mut self, index: usize) -> &mut Node<P, T> {
        &mut self.nodes[index]
    }

    /// Get the number of allocated nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the arena is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl<P: Point, T> Default for NodeArena<P, T> {
    fn default() -> Self {
        Self::new()
    }
}

/// External Arena Pattern: InMemoryLinker takes arena reference, doesn't own allocation.
///
/// # Architecture Decision: User controls node allocation, linker only handles linking
/// Benefits:
/// - Separation of concerns: allocation vs linking are independent
/// - Flexibility: user can pre-allocate, batch allocate, or use custom allocation strategies
/// - Reusability: same linker can work with different arena instances
/// - Testing: easier to create controlled test scenarios
///
/// # Usage pattern:
/// ```rust
/// let mut arena = NodeArena::new();
/// let node1 = arena.allocate(point, data);  // User allocates
/// let mut linker = InMemoryLinker::new(&mut arena);  // Linker borrows arena
/// // Tree algorithms use linker, don't see arena directly
/// ```
pub struct InMemoryLinker<'a, P: Point, T> {
    arena: &'a mut NodeArena<P, T>,
}

impl<'a, P: Point, T> InMemoryLinker<'a, P, T> {
    /// Create a new linker that operates on the given arena.
    pub fn new(arena: &'a mut NodeArena<P, T>) -> Self {
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
