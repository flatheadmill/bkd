//! Spatial search algorithms and tree construction.

use crate::spatial::{Point, SpatialPoint};
use crate::storage::NodeLinker;

/// Simple KD-tree insertion function demonstrating "tree tools" approach.
/// Takes a linker and inserts a node into the tree using alternating dimensions.
pub fn insert_node<P: Point, T, L: NodeLinker<P, T>>(
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

/// Generic spatial search function for KD-tree using NodeLinker abstraction.
/// Returns all nodes whose spatial data overlaps with or is within the query.
///
/// # Architecture
/// This implements the same spatial pruning logic as bbox.rs but generically:
/// - Uses NodeLinker abstraction to work with any storage backend (memory, files, compressed)
/// - Employs dimensional pruning: only visits subtrees that could contain overlapping results
/// - Alternates dimensions by depth: root splits on dim 0, children on dim 1, etc.
/// - For 4D bounding boxes: [xmin, ymin, xmax, ymax] cycle through dimensions 0,1,2,3
pub fn spatial_search<P: SpatialPoint, T, L: NodeLinker<P, T>>(
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
