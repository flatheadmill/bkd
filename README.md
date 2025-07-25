# BKD: Block KD-Tree Library for Geospatial Search

A Rust implementation of Lucene-inspired Block KD-Tree (BKD) for high-performance geospatial indexing and search, designed to integrate with Tantivy → pg_search → PostgreSQL.

## Project Overview

This library implements a complete geospatial search system based on Lucene's BKD algorithm, adapted for Rust with the following goals:

1. **Triangle Encoding**: 7-dimensional spatial encoding of triangles following Lucene's geometric shape handling
2. **Block KD-Tree**: Efficient spatial indexing with configurable block sizes and page-based storage
3. **Tantivy Integration**: Design as a library that can be integrated into Tantivy's search ecosystem
4. **PostgreSQL Pipeline**: Ultimate integration through pg_search into PostgreSQL for production database usage

## Documentation Standards

This project prioritizes **comprehensive documentation** for long-term maintainability:

- **Algorithm Documentation**: Each major function includes detailed comments explaining the mathematical and geometric reasoning
- **Lucene Correspondence**: Code comments explicitly reference equivalent Lucene implementations where applicable
- **Step-by-Step Explanations**: Complex algorithms (like triangle encoding) are broken down into numbered steps
- **Geometric Intuition**: Spatial algorithms include geometric explanations, not just code implementation
- **Context Preservation**: README updates accompany all significant changes to maintain project continuity

## Current Implementation Status

### ✅ Completed Components

#### Triangle Encoding System (`src/bin/triangle.rs`)
- **7-dimensional triangle representation** (28 bytes total)
- **Canonical form rotation** ensuring consistent encoding regardless of input vertex order
- **Edge flag preservation** for original polygon shape reconstruction
- **Sortable byte encoding** following Lucene's `NumericUtils` pattern
- **Complete round-trip encoding/decoding** with geometric orientation validation
- **Comprehensive algorithm documentation** with step-by-step encoding process explanation

#### KD-Tree Foundation (`src/tree.rs`, `src/arena.rs`)
- **Arena-based memory management** for efficient node allocation
- **Configurable block sizes** (currently set to 4 points per leaf)
- **Generic point structure** supporting n-dimensional coordinates
- **Tree insertion** with automatic splitting when blocks exceed capacity

#### Visualization and Testing (`src/print.rs`)
- **Tree structure visualization** with indented output showing internal/leaf nodes
- **Complete test suite** covering encoding round-trips and geometric operations
- **Multiple evolutionary implementations** in `src/attic/` showing development progression

### 🔄 In Progress

#### Research and Planning
- **Source code access strategy** for deep Lucene and Tantivy inspiration
- **Phase-by-phase development approach** following Tantivy's original methodology
- **Integration architecture** planning for minimal Tantivy modification

### 📋 Next Development Phases

#### Phase 1: Foundation Enhancement
- **Generic KD-Tree** supporting arbitrary dimensions beyond current 2D
- **Tessellation integration** using existing Rust tessellation crates
- **Triangle encoding refinement** with additional geometric primitives

#### Phase 2: Block KD-Tree Core
- **Lucene BKDWriter/BKDReader adaptation** studying Java implementation patterns
- **Page-based storage system** for disk-based indexing performance
- **Leaf node optimization** with configurable block sizes and splitting strategies
- **Tree construction algorithms** optimized for Rust memory patterns

#### Phase 3: Query Processing
- **Spatial query operations**: Point-in-polygon, bounding box intersection, range queries
- **Query optimization** leveraging Rust's zero-cost abstractions
- **Integration points** with Tantivy's existing query processing pipeline

## Architecture Decisions

### Memory Management
- **Arena allocation** chosen over Box/Rc for better cache locality and reduced fragmentation
- **Node IDs** instead of pointers for serialization compatibility and memory safety

### Geometric Encoding
- **7-dimensional approach** following Lucene's proven spatial indexing methodology
- **Sortable byte representation** ensuring proper ordering for tree construction and queries
- **Canonical form** eliminating duplicate representations of identical triangles

### Development Philosophy
- **Incremental approach** inspired by how Tantivy was originally built from Lucene concepts
- **Rust-native optimizations** while maintaining algorithmic compatibility with Lucene
- **Library-first design** enabling clean integration into existing search ecosystems
- **Documentation-driven development** ensuring long-term maintainability and knowledge transfer

## Code Organization

```
src/
├── main.rs              # Basic usage example and testing
├── tree.rs              # Core KD-Tree implementation with generic Point support
├── arena.rs             # Memory arena for efficient node management
├── print.rs             # Tree visualization and debugging utilities
├── bin/
│   └── triangle.rs      # Complete triangle encoding/decoding system (fully documented)
└── attic/              # Development history and experimental implementations
    ├── 00_kd.rs        # Basic KD-tree with recursive Box allocation
    ├── 01_block_kd.rs  # Generic block-based KD-tree with const generics
    └── 02_block_kd_print.rs # Block KD-tree with visualization
external/               # Source code references (not in git)
├── lucene/            # Apache Lucene source for algorithm reference
└── tantivy/           # Tantivy source for integration planning
```

## Usage Examples

### Triangle Encoding
```rust
use bkd::Triangle;

let triangle = Triangle::new(
    100, 200,  // Point A
    300, 150,  // Point B
    250, 400,  // Point C
    true, false, true  // Edge flags: AB, BC, CA from original shape
);

let encoded = encode_triangle(&triangle);  // 28 bytes
let decoded = decode_triangle(&encoded);   // Perfect round-trip
```

### KD-Tree Operations
```rust
use bkd::{KDTree, Point};

let mut tree = KDTree::new();
tree.insert(Point { x: 2.0, y: 3.0 });
tree.insert(Point { x: 5.0, y: 4.0 });
tree.print();  // Visualize tree structure
```

## Integration Strategy

The library is designed for **minimal invasive integration** into Tantivy:

1. **Library Independence**: Core BKD functionality works standalone
2. **Tantivy Adapter Layer**: Separate integration module handling Tantivy-specific concerns
3. **Query Integration**: Hooks into Tantivy's existing query processing without major architecture changes
4. **Incremental Adoption**: Can be introduced gradually alongside existing Tantivy features

## Performance Characteristics

- **Encoding**: O(1) triangle to 7D point conversion
- **Tree Construction**: O(n log n) with configurable block sizes for cache optimization
- **Spatial Queries**: O(log n) average case with geometric pruning
- **Memory Usage**: Arena allocation reduces fragmentation vs individual Box allocation

## Dependencies

- **Standard Library Only**: Core implementation uses no external dependencies
- **Future Dependencies**: Will integrate with tessellation crates and potentially Tantivy's existing infrastructure

## Development History

This codebase represents an evolutionary approach to BKD implementation:

1. **Basic KD-Tree** (`00_kd.rs`): Traditional recursive implementation
2. **Block KD-Tree** (`01_block_kd.rs`): Introduction of configurable block sizes
3. **Visualization** (`02_block_kd_print.rs`): Adding debugging and analysis tools
4. **Current Architecture**: Arena-based memory management with separation of concerns
5. **Documentation Phase**: Comprehensive algorithm documentation for long-term maintainability

## Contributing

The project follows an **incremental, research-driven approach**:

1. Study Lucene's implementation patterns deeply
2. Adapt algorithms to leverage Rust's strengths (memory safety, zero-cost abstractions)
3. Maintain compatibility with Tantivy's architecture philosophy
4. Prioritize performance while ensuring correctness
5. **Document everything** - algorithms, decisions, and architectural reasoning

### Documentation Guidelines

- **Algorithm Comments**: Explain the mathematical reasoning, not just the code
- **Lucene References**: Include links/references to corresponding Lucene implementations
- **Step Numbering**: Break complex processes into clearly numbered steps
- **Geometric Intuition**: For spatial algorithms, include ASCII diagrams or geometric explanations
- **Update README**: Significant changes should include corresponding README updates

---

*This library is part of a larger geospatial search pipeline: BKD → Tantivy → pg_search → PostgreSQL*
