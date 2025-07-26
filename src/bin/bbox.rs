/*
BOUNDING BOX KD-TREE DEMONSTRATION

This binary demonstrates creating and using KD-trees with bounding box data,
showing how to build spatial indexes for rectangular regions:

- BoundingBox spatial data with 4D coordinates (xmin, ymin, xmax, ymax)
- Storage-agnostic KD-tree algorithms using NodeLinker abstraction
- Spatial search with dimensional pruning for geographic/geometric queries
- SVG visualization of tree structure and search queries

The core library modules used:
- bkd::spatial - BoundingBox type implementing Point and SpatialPoint traits
- bkd::storage - NodeArena and InMemoryLinker for memory-based storage
- bkd::search - spatial_search, insert_node, and SVG visualization functions

This example demonstrates the fundamental bounding box use case for spatial
indexing, which forms the foundation for more complex geometric data types.
*/

// Import library modules
use bkd::search::{add_query_to_svg, tree_to_svg};
use bkd::spatial::{Point, SpatialPoint};
use bkd::storage::{Node, NodeLinker};
use bkd::{BoundingBox, InMemoryLinker, NodeArena, insert_node, spatial_search};

fn main() {
    println!("Bounding Box KD-tree demonstration with NodeLinker abstraction");
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

    // Generate SVG visualization
    println!("\n=== SVG Visualization Demo ===");
    let mut svg = tree_to_svg(&linker, Some(root), 800, 600);

    // Add query overlay for downtown search
    let svg_bounds = BoundingBox::new(0.0, 0.0, 12.0, 10.0); // Estimated bounds
    add_query_to_svg(&mut svg, &downtown_query, &svg_bounds, 800, 600);

    // Write SVG to file
    use std::fs::write;
    match write("kdtree_demo.svg", &svg) {
        Ok(_) => println!("SVG visualization saved to 'kdtree_demo.svg'"),
        Err(e) => println!("Failed to write SVG file: {}", e),
    }

    println!("You can open the SVG file in a web browser to view the tree visualization");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test the bbox program integration - verify it can run without errors
    #[test]
    fn test_bbox_program_runs() {
        // This test verifies the bbox program can execute its main functionality
        // without panicking. It's an integration test for the demo program itself.

        // Create the same data structures as main() but don't write to filesystem
        let mut arena = NodeArena::new();
        let store_ref = arena.allocate(BoundingBox::new(5.0, 5.0, 7.0, 7.0), 101);
        let house_ref = arena.allocate(BoundingBox::new(2.0, 2.0, 3.0, 3.0), 102);
        let park_ref = arena.allocate(BoundingBox::new(8.0, 1.0, 10.0, 2.0), 103);
        let school_ref = arena.allocate(BoundingBox::new(1.0, 8.0, 2.0, 9.0), 104);

        let mut linker = InMemoryLinker::new(&mut arena);

        // Build the spatial index
        let root = insert_node(&mut linker, None, store_ref, 0);
        insert_node(&mut linker, Some(root), house_ref, 0);
        insert_node(&mut linker, Some(root), park_ref, 0);
        insert_node(&mut linker, Some(root), school_ref, 0);

        // Test spatial searches work
        let downtown_query = BoundingBox::new(0.0, 0.0, 6.0, 6.0);
        let downtown_results = spatial_search(&linker, Some(root), &downtown_query, 0);
        assert!(
            !downtown_results.is_empty(),
            "Downtown search should find results"
        );

        let eastside_query = BoundingBox::new(7.0, 0.0, 12.0, 5.0);
        let eastside_results = spatial_search(&linker, Some(root), &eastside_query, 0);
        assert!(
            !eastside_results.is_empty(),
            "Eastside search should find results"
        );

        // Test SVG generation works (without writing to file)
        let svg = tree_to_svg(&linker, Some(root), 800, 600);
        assert!(svg.contains("<svg"), "Should generate valid SVG");
        assert!(svg.contains("</svg>"), "Should have closing SVG tag");

        // Test query overlay works
        let svg_bounds = BoundingBox::new(0.0, 0.0, 12.0, 10.0);
        let mut svg_with_query = svg.clone();
        add_query_to_svg(&mut svg_with_query, &downtown_query, &svg_bounds, 800, 600);
        assert!(
            svg_with_query.contains("query-box"),
            "Should add query overlay"
        );
    }
}
