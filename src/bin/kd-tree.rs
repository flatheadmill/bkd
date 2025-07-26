/*
BKD SPATIAL INDEXING DEMONSTRATION

This binary demonstrates the BKD spatial indexing library in action, showing:
- Storage-agnostic KD-tree algorithms with NodeLinker abstraction
- Spatial search with dimensional pruning
- Integration patterns for Tantivy components

The core library code has been modularized into:
- bkd::spatial - Point traits and BoundingBox implementation
- bkd::storage - NodeArena, NodeLinker trait, InMemoryLinker
- bkd::search - spatial_search and insert_node algorithms

This example shows how the same algorithms work with different storage backends
and provides a foundation for Tantivy integration.
*/

// Import library modules
use bkd::spatial::{Point, SpatialPoint};
use bkd::storage::{Node, NodeLinker};
use bkd::{BoundingBox, InMemoryLinker, NodeArena, insert_node, spatial_search};

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
