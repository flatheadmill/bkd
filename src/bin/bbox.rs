#[derive(Debug, Clone, PartialEq)]
pub struct BoundingBox {
    pub xmin: f64,
    pub ymin: f64,
    pub xmax: f64,
    pub ymax: f64,
}

impl BoundingBox {
    pub fn new(xmin: f64, ymin: f64, xmax: f64, ymax: f64) -> Self {
        BoundingBox { xmin, ymin, xmax, ymax }
    }

    // Get value for dimension (0=xmin, 1=ymin, 2=xmax, 3=ymax)
    pub fn get_dimension(&self, dim: usize) -> f64 {
        match dim {
            0 => self.xmin,
            1 => self.ymin,
            2 => self.xmax,
            3 => self.ymax,
            _ => panic!("Invalid dimension: {}", dim),
        }
    }

    // Check if this bounding box is fully within the query box
    pub fn is_within(&self, query: &BoundingBox) -> bool {
        self.xmin >= query.xmin && self.xmax <= query.xmax &&
        self.ymin >= query.ymin && self.ymax <= query.ymax
    }

    // Check if this bounding box overlaps with the query box
    pub fn overlaps(&self, query: &BoundingBox) -> bool {
        !(self.xmax < query.xmin || self.xmin > query.xmax ||
          self.ymax < query.ymin || self.ymin > query.ymax)
    }

    // Compute union of two bounding boxes (enclosing box)
    pub fn union(&self, other: &BoundingBox) -> BoundingBox {
        BoundingBox {
            xmin: self.xmin.min(other.xmin),
            ymin: self.ymin.min(other.ymin),
            xmax: self.xmax.max(other.xmax),
            ymax: self.ymax.max(other.ymax),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    // Enclosing bounding box for all individual boxes in this node's page
    pub enclosing_box: BoundingBox,
    // Page ID referencing external storage of individual bounding boxes
    pub page_id: u32,
    pub left: Option<Box<Node>>,
    pub right: Option<Box<Node>>,
}

impl Node {
    pub fn new(enclosing_box: BoundingBox, page_id: u32) -> Self {
        Node {
            enclosing_box,
            page_id,
            left: None,
            right: None,
        }
    }

    pub fn set_left(&mut self, node: Node) {
        self.left = Some(Box::new(node));
    }

    pub fn set_right(&mut self, node: Node) {
        self.right = Some(Box::new(node));
    }

    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }
}

#[derive(Debug)]
pub struct BoundingBoxKDTree {
    pub root: Option<Node>,
}

impl BoundingBoxKDTree {
    pub fn new() -> Self {
        BoundingBoxKDTree { root: None }
    }

    pub fn set_root(&mut self, node: Node) {
        self.root = Some(node);
    }

    pub fn get_root(&self) -> Option<&Node> {
        self.root.as_ref()
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn insert(&mut self, enclosing_box: BoundingBox, page_id: u32) {
        let new_node = Node::new(enclosing_box, page_id);
        if self.is_empty() {
            self.set_root(new_node);
        } else if let Some(root) = &mut self.root {
            Self::insert_recursive(0, root, new_node);
        }
    }

    fn insert_recursive(depth: usize, node: &mut Node, new_node: Node) {
        // Use depth % 4 for 4D cycling (xmin, ymin, xmax, ymax)
        let dimension = depth % 4;

        // Compare based on the current dimension
        let current_value = node.enclosing_box.get_dimension(dimension);
        let new_value = new_node.enclosing_box.get_dimension(dimension);

        if new_value < current_value {
            match &mut node.left {
                Some(left_child) => {
                    Self::insert_recursive(depth + 1, left_child, new_node);
                }
                None => {
                    node.left = Some(Box::new(new_node));
                }
            }
        } else {
            match &mut node.right {
                Some(right_child) => {
                    Self::insert_recursive(depth + 1, right_child, new_node);
                }
                None => {
                    node.right = Some(Box::new(new_node));
                }
            }
        }
    }

    // Query for all pages containing bounding boxes within the query box
    pub fn query_within(&self, query_box: &BoundingBox) -> Vec<u32> {
        let mut results = Vec::new();
        if let Some(root) = &self.root {
            Self::query_recursive(0, root, query_box, &mut results);
        }
        results
    }

    fn query_recursive(depth: usize, node: &Node, query_box: &BoundingBox, results: &mut Vec<u32>) {
        // Check if node's enclosing box is fully within query box
        if node.enclosing_box.is_within(query_box) {
            // All individual boxes in this page are guaranteed to be within query
            results.push(node.page_id);
        } else if node.enclosing_box.overlaps(query_box) {
            // Partial overlap - need to inspect individual boxes in the page
            // For now, we collect the page_id (in real implementation, you'd fetch and filter)
            results.push(node.page_id);
        }
        // If no overlap at all, we skip this node entirely (pruning)

        // Determine which children to visit based on current dimension split
        let dimension = depth % 4;
        let split_value = node.enclosing_box.get_dimension(dimension);

        // Check if left subtree could contain relevant results
        let query_min = match dimension {
            0 => query_box.xmin,
            1 => query_box.ymin,
            2 => query_box.xmax,
            3 => query_box.ymax,
            _ => unreachable!(),
        };

        let query_max = match dimension {
            0 => query_box.xmax,
            1 => query_box.ymax,
            2 => query_box.xmax,
            3 => query_box.ymax,
            _ => unreachable!(),
        };

        // Recurse into left child if query could overlap left subspace
        if let Some(left) = &node.left {
            if query_min <= split_value {
                Self::query_recursive(depth + 1, left, query_box, results);
            }
        }

        // Recurse into right child if query could overlap right subspace
        if let Some(right) = &node.right {
            if query_max >= split_value {
                Self::query_recursive(depth + 1, right, query_box, results);
            }
        }
    }

    // Generate SVG visualization of the tree
    pub fn to_svg(&self, width: u32, height: u32) -> String {
        let mut svg = String::new();

        // Calculate bounds to scale the coordinates
        let bounds = self.calculate_bounds();

        // SVG header
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
    .page-text {{ font-family: Arial; font-size: 12px; fill: black; }}
    .query-box {{ fill: rgba(255, 255, 0, 0.3); stroke: black; stroke-width: 1; stroke-dasharray: 5,5; }}
    .background {{ fill: white; }}
</style>
<rect x="0" y="0" width="{}" height="{}" class="background" />
"#,
            width, height, width, height
        ));

        if let Some(root) = &self.root {
            self.render_node_svg(root, 0, &bounds, width, height, &mut svg);
        }

        svg.push_str("</svg>");
        svg
    }

    fn calculate_bounds(&self) -> BoundingBox {
        if let Some(root) = &self.root {
            let mut bounds = root.enclosing_box.clone();
            self.expand_bounds(root, &mut bounds);

            // Add some padding
            let padding = ((bounds.xmax - bounds.xmin) + (bounds.ymax - bounds.ymin)) * 0.1;
            bounds.xmin -= padding;
            bounds.ymin -= padding;
            bounds.xmax += padding;
            bounds.ymax += padding;

            bounds
        } else {
            BoundingBox::new(0.0, 0.0, 100.0, 100.0)
        }
    }

    fn expand_bounds(&self, node: &Node, bounds: &mut BoundingBox) {
        *bounds = bounds.union(&node.enclosing_box);

        if let Some(left) = &node.left {
            self.expand_bounds(left, bounds);
        }
        if let Some(right) = &node.right {
            self.expand_bounds(right, bounds);
        }
    }

    fn render_node_svg(&self, node: &Node, depth: usize, bounds: &BoundingBox,
                      svg_width: u32, svg_height: u32, svg: &mut String) {
        // Transform coordinates from world space to SVG space
        let x1 = ((node.enclosing_box.xmin - bounds.xmin) / (bounds.xmax - bounds.xmin)) * svg_width as f64;
        let y1 = ((bounds.ymax - node.enclosing_box.ymax) / (bounds.ymax - bounds.ymin)) * svg_height as f64; // Flip Y
        let x2 = ((node.enclosing_box.xmax - bounds.xmin) / (bounds.xmax - bounds.xmin)) * svg_width as f64;
        let y2 = ((bounds.ymax - node.enclosing_box.ymin) / (bounds.ymax - bounds.ymin)) * svg_height as f64; // Flip Y

        let width = x2 - x1;
        let height = y2 - y1;

        // Draw rectangle
        svg.push_str(&format!(
            r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" class="bbox depth-{}" />
"#,
            x1, y1, width, height, depth % 8
        ));

        // Add page ID text
        let text_x = x1 + width / 2.0;
        let text_y = y1 + height / 2.0;
        svg.push_str(&format!(
            r#"<text x="{:.1}" y="{:.1}" text-anchor="middle" dominant-baseline="middle" class="page-text">P{}</text>
"#,
            text_x, text_y, node.page_id
        ));

        // Recursively render children
        if let Some(left) = &node.left {
            self.render_node_svg(left, depth + 1, bounds, svg_width, svg_height, svg);
        }
        if let Some(right) = &node.right {
            self.render_node_svg(right, depth + 1, bounds, svg_width, svg_height, svg);
        }
    }

    // Add a method to render a query box overlay
    pub fn add_query_to_svg(&self, svg: &mut String, query_box: &BoundingBox,
                           bounds: &BoundingBox, svg_width: u32, svg_height: u32) {
        // Transform query coordinates to SVG space
        let x1 = ((query_box.xmin - bounds.xmin) / (bounds.xmax - bounds.xmin)) * svg_width as f64;
        let y1 = ((bounds.ymax - query_box.ymax) / (bounds.ymax - bounds.ymin)) * svg_height as f64;
        let x2 = ((query_box.xmax - bounds.xmin) / (bounds.xmax - bounds.xmin)) * svg_width as f64;
        let y2 = ((bounds.ymax - query_box.ymin) / (bounds.ymax - bounds.ymin)) * svg_height as f64;

        let width = x2 - x1;
        let height = y2 - y1;

        // Insert query box before closing </svg> tag
        let query_rect = format!(
            r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" class="query-box" />
<text x="{:.1}" y="{:.1}" text-anchor="middle" class="page-text">Query</text>
"#,
            x1, y1, width, height, x1 + width / 2.0, y1 - 5.0
        );

        if let Some(pos) = svg.rfind("</svg>") {
            svg.insert_str(pos, &query_rect);
        }
    }

    // Convenience method to generate SVG with query overlay
    pub fn to_svg_with_query(&self, width: u32, height: u32, query_box: &BoundingBox) -> String {
        let bounds = self.calculate_bounds();
        let mut svg = self.to_svg(width, height);
        self.add_query_to_svg(&mut svg, query_box, &bounds, width, height);
        svg
    }
}

fn main() {
    println!("Testing 4D Bounding Box KD-Tree");

    let mut tree = BoundingBoxKDTree::new();

    // Insert some bounding boxes with page IDs
    tree.insert(BoundingBox::new(0.0, 0.0, 10.0, 10.0), 1); // Root
    tree.insert(BoundingBox::new(-5.0, -5.0, 5.0, 5.0), 2); // Should go left
    tree.insert(BoundingBox::new(5.0, 5.0, 15.0, 15.0), 3); // Should go right
    tree.insert(BoundingBox::new(-10.0, 2.0, -2.0, 8.0), 4); // Further left
    tree.insert(BoundingBox::new(12.0, -3.0, 18.0, 3.0), 5); // Further right

    println!("Tree structure: {:#?}", tree);

    // Query for bounding boxes within a specific region
    let query = BoundingBox::new(-1.0, -1.0, 12.0, 12.0);
    let matching_pages = tree.query_within(&query);

    println!("\nQuery box: {:?}", query);
    println!("Matching page IDs: {:?}", matching_pages);

    // Generate SVG visualization
    let svg_content = tree.to_svg_with_query(800, 600, &query);

    // Write SVG to file
    use std::fs;
    match fs::write("tree_visualization.svg", &svg_content) {
        Ok(_) => println!("\nSVG saved to tree_visualization.svg"),
        Err(e) => println!("Error writing SVG: {}", e),
    }

    // Also print SVG content for immediate viewing
    println!("\nSVG Content:");
    println!("{}", svg_content);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounding_box_within() {
        let inner = BoundingBox::new(2.0, 3.0, 4.0, 5.0);
        let outer = BoundingBox::new(1.0, 2.0, 5.0, 6.0);
        assert!(inner.is_within(&outer));
        assert!(!outer.is_within(&inner));
    }

    #[test]
    fn test_bounding_box_overlaps() {
        let box1 = BoundingBox::new(0.0, 0.0, 5.0, 5.0);
        let box2 = BoundingBox::new(3.0, 3.0, 8.0, 8.0);
        let box3 = BoundingBox::new(10.0, 10.0, 15.0, 15.0);

        assert!(box1.overlaps(&box2));
        assert!(!box1.overlaps(&box3));
    }

    #[test]
    fn test_tree_insertion_and_query() {
        let mut tree = BoundingBoxKDTree::new();

        tree.insert(BoundingBox::new(0.0, 0.0, 10.0, 10.0), 1);
        tree.insert(BoundingBox::new(2.0, 2.0, 4.0, 4.0), 2);

        let query = BoundingBox::new(1.0, 1.0, 5.0, 5.0);
        let results = tree.query_within(&query);

        assert!(results.contains(&1));
        assert!(results.contains(&2));
    }
}
