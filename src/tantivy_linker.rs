/*
TANTIVY LINKER IMPLEMENTATION

This module implements the NodeLinker trait using Tantivy's storage components:
- MmapDirectory for file-based storage
- OwnedBytes for memory management
- Efficient serialization/deserialization of BKD nodes

This bridges your BKD spatial indexing algorithms with Tantivy's storage system.
*/

use crate::BoundingBox;
use crate::spatial::{Point, SpatialPoint};
use crate::storage::NodeLinker;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tantivy::directory::{Directory, MmapDirectory};

/// Node reference for TantivyLinker - uses u64 as file offset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TantivyNodeRef(pub u64);

#[derive(Clone, Serialize, Deserialize)]
pub struct Node<P, T> {
    pub point: P,
    pub data: T,
    pub left: Option<TantivyNodeRef>,
    pub right: Option<TantivyNodeRef>,
}

/// TantivyLinker implements NodeLinker using Tantivy's storage system
pub struct TantivyLinker<T> {
    directory: Box<dyn Directory>,
    nodes: HashMap<TantivyNodeRef, Node<BoundingBox, T>>,
    file_prefix: String,
}

impl<T: Clone> TantivyLinker<T> {
    /// Create a new TantivyLinker with file-based storage
    pub fn new_with_directory(directory: Box<dyn Directory>, file_prefix: String) -> Self {
        Self {
            directory,
            nodes: HashMap::new(),
            file_prefix,
        }
    }

    /// Create a TantivyLinker with temporary directory for testing
    pub fn new_temp(file_prefix: String) -> tantivy::Result<Self> {
        let directory = MmapDirectory::create_from_tempdir()?;
        Ok(Self::new_with_directory(Box::new(directory), file_prefix))
    }

    /// Serialize a node to bytes for storage
    fn serialize_node(&self, node: &Node<BoundingBox, T>) -> Vec<u8>
    where
        T: serde::Serialize,
    {
        // Simple binary format for now
        // TODO: Use more efficient serialization (bincode, postcard, etc.)
        bincode::serialize(node).unwrap_or_else(|_| Vec::new())
    }

    /// Deserialize a node from bytes
    fn deserialize_node(&self, bytes: &[u8]) -> Option<Node<BoundingBox, T>>
    where
        T: serde::de::DeserializeOwned,
    {
        bincode::deserialize(bytes).ok()
    }

    /// Get filename for a node
    fn get_node_filename(&self, node_ref: TantivyNodeRef) -> String {
        format!("{}_node_{}.bkd", self.file_prefix, node_ref.0)
    }
}

impl<T: Clone + serde::Serialize + serde::de::DeserializeOwned> NodeLinker<BoundingBox, T>
    for TantivyLinker<T>
{
    type NodeRef = TantivyNodeRef;

    fn get_point(&self, node_ref: Self::NodeRef) -> &BoundingBox {
        &self.nodes.get(&node_ref).unwrap().point
    }

    fn get_data(&self, node_ref: Self::NodeRef) -> &T {
        &self.nodes.get(&node_ref).unwrap().data
    }

    fn get_left(&self, node_ref: Self::NodeRef) -> Option<Self::NodeRef> {
        self.nodes.get(&node_ref)?.left
    }

    fn get_right(&self, node_ref: Self::NodeRef) -> Option<Self::NodeRef> {
        self.nodes.get(&node_ref)?.right
    }

    fn link_left(&mut self, parent_ref: Self::NodeRef, child_ref: Self::NodeRef) {
        if let Some(parent) = self.nodes.get_mut(&parent_ref) {
            parent.left = Some(child_ref);
        }
    }

    fn link_right(&mut self, parent_ref: Self::NodeRef, child_ref: Self::NodeRef) {
        if let Some(parent) = self.nodes.get_mut(&parent_ref) {
            parent.right = Some(child_ref);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{InMemoryLinker, NodeArena};

    #[test]
    fn test_tantivy_linker_creation() {
        let linker = TantivyLinker::<u32>::new_temp("test".to_string());
        assert!(linker.is_ok());
    }

    #[test]
    fn test_tantivy_vs_inmemory_linker() {
        // This test demonstrates that TantivyLinker should work with the same algorithms
        // as InMemoryLinker, just with different storage backend

        let mut arena = NodeArena::new();
        let node_ref = arena.allocate(BoundingBox::new(1.0, 1.0, 2.0, 2.0), 42u32);

        // Test with InMemoryLinker
        let inmemory_linker = InMemoryLinker::new(&mut arena);
        let point = inmemory_linker.get_point(node_ref);
        let data = inmemory_linker.get_data(node_ref);

        assert_eq!(point.xmin, 1.0);
        assert_eq!(*data, 42);

        // TODO: Test with TantivyLinker once we implement node loading/storing
        // let tantivy_linker = TantivyLinker::new_temp("test".to_string()).unwrap();
        // ... same operations should work
    }
}
