#[derive(Debug, Clone)]
pub struct Node<T> {
    pub points: Vec<i32>,
    pub value: T,
    pub left: Option<Box<Node<T>>>,
    pub right: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    // Create a new node with values and a generic value
    pub fn new(values: Vec<i32>, value: T) -> Self {
        Node {
            points: values,
            value,
            left: None,
            right: None,
        }
    }

    // Create a new node with a single point value and a generic value
    pub fn new_single(point_value: i32, value: T) -> Self {
        Node {
            points: vec![point_value],
            left: None,
            right: None,
            value,
        }
    }

    // Add a child to the left
    pub fn set_left(&mut self, node: Node<T>) {
        self.left = Some(Box::new(node));
    }

    // Add a child to the right
    pub fn set_right(&mut self, node: Node<T>) {
        self.right = Some(Box::new(node));
    }

    // Add a value to this node's points array
    pub fn add_point(&mut self, point_value: i32) {
        self.points.push(point_value);
    }

    // Check if this is a leaf node
    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }
}

pub struct KDTree<T> {
    pub root: Option<Node<T>>,
}

impl<T> KDTree<T>
where
    T: Clone + Ord,
{
    // Create a new tree
    pub fn new() -> Self {
        KDTree { root: None }
    }

    // Set the root node
    pub fn set_root(&mut self, node: Node<T>) {
        self.root = Some(node);
    }

    // Get the root node
    pub fn get_root(&self) -> Option<&Node<T>> {
        self.root.as_ref()
    }

    // Check if the tree is empty
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn insert(&mut self, points: Vec<i32>, value: T) {
        let new_node = Node::new(points.clone(), value);
        if self.is_empty() {
            self.set_root(new_node);
        } else {
            let depth = 0; // Start at depth 0
            // Compare the vector value of the new node with the vector value of the root node
            // and decide where to insert it (left or right)
            if let Some(root) = &mut self.root {
                Self::insert_recursive(depth, root, new_node);
            }
        }
    }

    fn insert_recursive(depth: i32, node: &mut Node<T>, new_node: Node<T>) {
        // Use the depth to determine which dimension to compare
        let dimension = (depth % new_node.points.len() as i32) as usize;
        // Compare the new node's point in the current dimension with the current node's point.
        // If the new node's point is less than the current node's point, go left
        // Otherwise, go right.
        if new_node.points[dimension] < node.points[dimension] {
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
}

impl<T> Default for KDTree<T>
where
    T: Clone + Ord,
{
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    // Print hello, world!
    println!("Hello, world!");

    // Create a new KDTree with String as the generic type
    let mut tree = KDTree::<String>::new();

    // Example usage
    let mut root = Node::new(vec![1, 2, 3], "root".to_string());

    let left_child = Node::new(vec![4, 5], "left".to_string());
    let right_child = Node::new_single(6, "right".to_string());

    root.set_left(left_child);
    root.set_right(right_child);

    tree.set_root(root);

    println!("Root node: {:?}", tree.get_root());
    println!("Is leaf: {}", tree.get_root().map(|n| n.is_leaf()).unwrap_or(false));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = Node::new(vec![1, 2, 3], "test".to_string());
        assert_eq!(node.points, vec![1, 2, 3]);
        assert_eq!(node.value, "test");
        assert!(node.is_leaf());
    }

    #[test]
    fn test_node_single_creation() {
        let node = Node::new_single(5, 42);
        assert_eq!(node.points, vec![5]);
        assert_eq!(node.value, 42);
        assert!(node.is_leaf());
    }

    #[test]
    fn test_tree_empty() {
        let tree: KDTree<String> = KDTree::new();
        assert!(tree.is_empty());
        assert!(tree.get_root().is_none());
    }

    #[test]
    fn test_tree_insert() {
        let mut tree = KDTree::<String>::new();
        tree.insert(vec![5, 5], "root".to_string());

        assert!(!tree.is_empty());
        let root = tree.get_root().unwrap();
        assert_eq!(root.points, vec![5, 5]);
        assert_eq!(root.value, "root");
    }

    #[test]
    fn test_tree_multiple_inserts() {
        let mut tree = KDTree::<i32>::new();

        tree.insert(vec![5, 5], 1);
        tree.insert(vec![2, 3], 2);  // Should go left (2 < 5 in dimension 0)
        tree.insert(vec![8, 7], 3);  // Should go right (8 > 5 in dimension 0)

        let root = tree.get_root().unwrap();
        assert_eq!(root.value, 1);

        // Check left child
        let left = root.left.as_ref().unwrap();
        assert_eq!(left.points, vec![2, 3]);
        assert_eq!(left.value, 2);

        // Check right child
        let right = root.right.as_ref().unwrap();
        assert_eq!(right.points, vec![8, 7]);
        assert_eq!(right.value, 3);
    }

    #[test]
    fn test_add_point() {
        let mut node = Node::new(vec![1, 2], "test".to_string());
        node.add_point(3);
        assert_eq!(node.points, vec![1, 2, 3]);
    }
}
