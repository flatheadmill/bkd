//! Spatial search algorithms and tree construction.

use crate::spatial::{BoundingBox, Point, SpatialPoint};
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

/// Generate SVG visualization of a KD-tree using NodeLinker abstraction.
/// Specifically works with BoundingBox spatial data for proper bounds calculation.
///
/// # Architecture
/// This provides tree visualization for debugging and understanding:
/// - Uses NodeLinker to traverse tree structure without knowing storage details
/// - Colors nodes by depth to show KD-tree splitting pattern
/// - Shows spatial relationships between bounding boxes
/// - Displays data IDs for each node
pub fn tree_to_svg<T, L: NodeLinker<BoundingBox, T>>(
    linker: &L,
    root: Option<L::NodeRef>,
    width: u32,
    height: u32,
) -> String
where
    T: std::fmt::Display,
{
    let mut svg = String::new();

    // Calculate bounds to scale the coordinates
    let bounds = if let Some(root_ref) = root {
        calculate_tree_bounds(linker, root_ref)
    } else {
        // Default bounds if no tree
        return format!(
            r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
<text x="50%" y="50%" text-anchor="middle" dominant-baseline="middle">Empty Tree</text>
</svg>"#,
            width, height
        );
    };

    // SVG header with styling
    svg.push_str(&format!(
        r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
<style>
    .bbox {{ fill: none; stroke-width: 2; }}
    .depth-0 {{ stroke: red; }}
    .depth-1 {{ stroke: blue; }}
    .depth-2 {{ stroke: green; }}
    .depth-3 {{ stroke: purple; }}
    .depth-4 {{ stroke: orange; }}
    .depth-5 {{ stroke: brown; }}
    .depth-6 {{ stroke: pink; }}
    .depth-7 {{ stroke: gray; }}
    .data-text {{ font-family: Arial; font-size: 12px; fill: black; }}
    .query-box {{ fill: rgba(255, 255, 0, 0.3); stroke: black; stroke-width: 1; stroke-dasharray: 5,5; }}
    .background {{ fill: white; }}
</style>
<rect x="0" y="0" width="{}" height="{}" class="background" />
"#,
        width, height, width, height
    ));

    if let Some(root_ref) = root {
        render_tree_node_svg(linker, root_ref, 0, &bounds, width, height, &mut svg);
    }

    svg.push_str("</svg>");
    svg
}

/// Calculate the bounding box that contains all nodes in the tree
fn calculate_tree_bounds<T, L: NodeLinker<BoundingBox, T>>(
    linker: &L,
    root: L::NodeRef,
) -> BoundingBox {
    let root_point = linker.get_point(root);
    let mut bounds = root_point.clone();

    expand_tree_bounds(linker, root, &mut bounds);

    // Add padding - expand bounds by 10%
    let mut padded_bounds = bounds.clone();
    for dim in 0..bounds.dimensions() {
        let coord = bounds.get_dimension(dim);
        let padding = coord.abs() * 0.1 + 1.0; // At least 1.0 unit padding

        // For min dimensions (0, 1), subtract padding
        // For max dimensions (2, 3), add padding
        let new_coord = if dim < 2 {
            coord - padding
        } else {
            coord + padding
        };
        padded_bounds = padded_bounds.with_dimension(dim, new_coord);
    }

    padded_bounds
}

/// Expand bounds to include all nodes in the subtree
fn expand_tree_bounds<T, L: NodeLinker<BoundingBox, T>>(
    linker: &L,
    node: L::NodeRef,
    bounds: &mut BoundingBox,
) {
    let node_point = linker.get_point(node);

    // Use the union method to expand bounds
    *bounds = bounds.union(&node_point);

    // Recursively expand for children
    if let Some(left_child) = linker.get_left(node) {
        expand_tree_bounds(linker, left_child, bounds);
    }
    if let Some(right_child) = linker.get_right(node) {
        expand_tree_bounds(linker, right_child, bounds);
    }
}

/// Render a single node and its children recursively
fn render_tree_node_svg<T, L: NodeLinker<BoundingBox, T>>(
    linker: &L,
    node: L::NodeRef,
    depth: usize,
    bounds: &BoundingBox,
    svg_width: u32,
    svg_height: u32,
    svg: &mut String,
) where
    T: std::fmt::Display,
{
    let node_point = linker.get_point(node);

    // 4D bounding box format: [xmin, ymin, xmax, ymax]
    let xmin = node_point.get_dimension(0);
    let ymin = node_point.get_dimension(1);
    let xmax = node_point.get_dimension(2);
    let ymax = node_point.get_dimension(3);

    let bounds_xmin = bounds.get_dimension(0);
    let bounds_ymin = bounds.get_dimension(1);
    let bounds_xmax = bounds.get_dimension(2);
    let bounds_ymax = bounds.get_dimension(3);

    // Transform coordinates from world space to SVG space
    let x1 = ((xmin - bounds_xmin) / (bounds_xmax - bounds_xmin)) * svg_width as f64;
    let y1 = ((bounds_ymax - ymax) / (bounds_ymax - bounds_ymin)) * svg_height as f64; // Flip Y
    let x2 = ((xmax - bounds_xmin) / (bounds_xmax - bounds_xmin)) * svg_width as f64;
    let y2 = ((bounds_ymax - ymin) / (bounds_ymax - bounds_ymin)) * svg_height as f64; // Flip Y

    let width = x2 - x1;
    let height = y2 - y1;

    // Draw rectangle
    svg.push_str(&format!(
        r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" class="bbox depth-{}" />
"#,
        x1,
        y1,
        width,
        height,
        depth % 8
    ));

    // Add data text
    let text_x = x1 + width / 2.0;
    let text_y = y1 + height / 2.0;
    let data_ref = linker.get_data(node);
    svg.push_str(&format!(
        r#"<text x="{:.1}" y="{:.1}" text-anchor="middle" dominant-baseline="middle" class="data-text">{}</text>
"#,
        text_x, text_y, data_ref
    ));

    // Recursively render children
    if let Some(left_child) = linker.get_left(node) {
        render_tree_node_svg(
            linker,
            left_child,
            depth + 1,
            bounds,
            svg_width,
            svg_height,
            svg,
        );
    }
    if let Some(right_child) = linker.get_right(node) {
        render_tree_node_svg(
            linker,
            right_child,
            depth + 1,
            bounds,
            svg_width,
            svg_height,
            svg,
        );
    }
}

/// Add a query box overlay to existing SVG
/// Call this after tree_to_svg to highlight the search area
pub fn add_query_to_svg(
    svg: &mut String,
    query: &BoundingBox,
    bounds: &BoundingBox,
    svg_width: u32,
    svg_height: u32,
) {
    // 4D bounding box format for query and bounds
    let query_xmin = query.get_dimension(0);
    let query_ymin = query.get_dimension(1);
    let query_xmax = query.get_dimension(2);
    let query_ymax = query.get_dimension(3);

    let bounds_xmin = bounds.get_dimension(0);
    let bounds_ymin = bounds.get_dimension(1);
    let bounds_xmax = bounds.get_dimension(2);
    let bounds_ymax = bounds.get_dimension(3);

    // Transform query coordinates to SVG space
    let x1 = ((query_xmin - bounds_xmin) / (bounds_xmax - bounds_xmin)) * svg_width as f64;
    let y1 = ((bounds_ymax - query_ymax) / (bounds_ymax - bounds_ymin)) * svg_height as f64;
    let x2 = ((query_xmax - bounds_xmin) / (bounds_xmax - bounds_xmin)) * svg_width as f64;
    let y2 = ((bounds_ymax - query_ymin) / (bounds_ymax - bounds_ymin)) * svg_height as f64;

    let width = x2 - x1;
    let height = y2 - y1;

    // Insert query box before closing </svg> tag
    let closing_tag_pos = svg.rfind("</svg>").unwrap();
    let query_rect = format!(
        r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" class="query-box" />
<text x="{:.1}" y="{:.1}" text-anchor="middle" class="data-text">Query</text>
"#,
        x1,
        y1,
        width,
        height,
        x1 + width / 2.0,
        y1 + height / 2.0
    );

    svg.insert_str(closing_tag_pos, &query_rect);
}
