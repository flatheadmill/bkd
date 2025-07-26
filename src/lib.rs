//! BKD Spatial Indexing Library
//!
//! A Rust implementation of Block KD-Tree (BKD) spatial indexing, inspired by Apache Lucene
//! and designed for integration with Tantivy search engine.
//!
//! # Overview
//!
//! This library provides efficient spatial indexing for multi-dimensional data using the BKD
//! (Block KD-Tree) algorithm. Key features include:
//!
//! - **Generic spatial indexing**: Works with any type implementing the `Point` trait
//! - **Storage abstraction**: `NodeLinker` trait enables multiple storage backends
//! - **Dimensional pruning**: Optimized spatial search with geometric pruning
//! - **Tantivy integration**: Designed to work with Tantivy's storage primitives
//!
//! # Architecture
//!
//! The library uses a modular architecture with clear separation of concerns:
//!
//! ```text
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │ Spatial Traits  │    │ Storage         │    │ Search          │
//! │ Point           │    │ NodeLinker      │    │ spatial_search  │
//! │ SpatialPoint    │    │ NodeArena       │    │ insert_node     │
//! └─────────────────┘    └─────────────────┘    └─────────────────┘
//! ```
//!
//! # Basic Usage
//!
//! ```rust
//! use bkd::{BoundingBox, NodeArena, InMemoryLinker, spatial_search, insert_node};
//!
//! // Create a spatial index for bounding boxes
//! let mut arena = NodeArena::new();
//! let bbox1 = arena.allocate(BoundingBox::new(1.0, 1.0, 2.0, 2.0), "location1");
//! let bbox2 = arena.allocate(BoundingBox::new(3.0, 3.0, 4.0, 4.0), "location2");
//!
//! let mut linker = InMemoryLinker::new(&mut arena);
//!
//! // Build spatial index
//! let root = insert_node(&mut linker, None, bbox1, 0);
//! insert_node(&mut linker, Some(root), bbox2, 0);
//!
//! // Search for overlapping regions
//! let query = BoundingBox::new(0.5, 0.5, 1.5, 1.5);
//! let results = spatial_search(&linker, Some(root), &query, 0);
//! ```

pub mod search;
pub mod spatial;
pub mod storage;

// Tantivy integration module (optional)
#[cfg(feature = "tantivy")]
pub mod tantivy_linker;

// Re-export key types for convenience
pub use search::{insert_node, spatial_search};
pub use spatial::{BoundingBox, Point, SpatialPoint};
pub use storage::{InMemoryLinker, NodeArena, NodeLinker};
