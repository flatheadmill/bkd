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
- **Development Diary**: Technical decisions, research findings, and architectural discussions tracked in [`diary.md`](diary.md)
- **Lucene Correspondence**: Code comments explicitly reference equivalent Lucene implementations where applicable
- **Context Preservation**: README and diary updates accompany all significant changes

## Current Implementation Status

### ✅ Completed Components

#### Triangle Encoding System (`src/bin/triangle.rs`)
- **7-dimensional triangle representation** (28 bytes total) - **100% Lucene compatible**
- **Canonical form rotation** ensuring consistent encoding regardless of input vertex order
- **Edge flag preservation** for original polygon shape reconstruction
- **Sortable byte encoding** following Lucene's `NumericUtils` pattern
- **Complete round-trip encoding/decoding** with geometric orientation validation

#### KD-Tree Foundation (`src/tree.rs`, `src/arena.rs`)
- **Arena-based memory management** for efficient node allocation
- **Configurable block sizes** (currently set to 4 points per leaf)
- **Generic point structure** supporting n-dimensional coordinates
- **Tree insertion** with automatic splitting when blocks exceed capacity

### 🔄 In Progress

- **Source code analysis** of Lucene BKD implementation (see [`diary.md`](diary.md))
- **Tantivy integration planning** based on architecture research
- **Generic KD-Tree** extension to support arbitrary dimensions

### 📋 Next Development Phases

#### Phase 1: Foundation Enhancement
- **Generic KD-Tree** supporting arbitrary dimensions beyond current 2D
- **BKD Configuration** system matching Lucene's parameters
- **Tessellation integration** using existing Rust tessellation crates

#### Phase 2: Block KD-Tree Core
- **BKDWriter/BKDReader** implementation following Lucene patterns
- **Page-based storage** system for disk-based indexing performance
- **Tree construction** algorithms optimized for Rust memory patterns

#### Phase 3: Tantivy Integration
- **Spatial field types** (`LatLonShape`, `XYShape`) for Tantivy schema
- **Spatial queries** integrated with Tantivy's query processing pipeline
- **Performance optimization** leveraging Tantivy's FastField infrastructure

## Architecture Decisions

### Core Design Principles
- **Library-first design**: Standalone BKD implementation with clean Tantivy integration layer
- **Lucene compatibility**: Maintain algorithmic compatibility for proven performance
- **Arena allocation**: Chosen for better cache locality and reduced fragmentation
- **Documentation-driven**: Comprehensive documentation for long-term maintainability

### Technical Compatibility
- **Triangle Encoding**: 100% compatible with Lucene's `ShapeField.encodeTriangle()`
- **BKD Configuration**: Target `BKDConfig(7, 4, 4, 512)` matching Lucene shapes
- **Sortable Bytes**: Identical to Lucene's `NumericUtils.intToSortableBytes()`

## Code Organization

```
src/
├── main.rs              # Basic usage example and testing
├── tree.rs              # Core KD-Tree implementation with generic Point support
├── arena.rs             # Memory arena for efficient node management
├── print.rs             # Tree visualization and debugging utilities
├── bin/
│   └── triangle.rs      # Complete triangle encoding/decoding system
└── attic/              # Experimental implementations and development history
external/               # Source code references (not in git)
├── lucene/            # Apache Lucene source for algorithm reference
└── tantivy/           # Tantivy source for integration planning
diary.md               # Development diary with technical decisions and research
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

## Performance Characteristics

- **Encoding**: O(1) triangle to 7D point conversion
- **Tree Construction**: O(n log n) with configurable block sizes for cache optimization
- **Spatial Queries**: O(log n) average case with geometric pruning
- **Memory Usage**: Arena allocation reduces fragmentation vs individual Box allocation

## Contributing

The project follows an **incremental, research-driven approach**:

1. Study Lucene's implementation patterns deeply
2. Adapt algorithms to leverage Rust's strengths (memory safety, zero-cost abstractions)
3. Maintain compatibility with Tantivy's architecture philosophy
4. **Document everything** - see [`diary.md`](diary.md) for detailed technical discussions

### Documentation Guidelines

- **Algorithm Comments**: Explain the mathematical reasoning, not just the code
- **Diary Entries**: Record architectural decisions and research findings
- **Lucene References**: Include links/references to corresponding Lucene implementations
- **README Updates**: Keep README concise, move detailed analysis to diary

---

*This library is part of a larger geospatial search pipeline: BKD → Tantivy → pg_search → PostgreSQL*

*For detailed technical analysis and architectural decisions, see [`diary.md`](diary.md)*
