# BKD + Tantivy Development Guide

This document explains how to work with the multi-folder VSCode workspace for developing the BKD spatial indexing library alongside Tantivy integration.

## Workspace Structure

The workspace consists of three main folders:

```
bkd-tantivy.code-workspace
├── BKD Library (Main)     # Our spatial indexing library
├── Tantivy Source         # Full Tantivy source for integration
└── Lucene Reference       # Lucene BKD implementation reference
```

## Getting Started

### 1. Open the Workspace
```bash
# From the bkd directory:
code bkd-tantivy.code-workspace
```

### 2. Install Recommended Extensions
VSCode will prompt you to install recommended extensions:
- **rust-analyzer**: Essential Rust language support
- **Even Better TOML**: Better Cargo.toml editing
- **crates**: Dependency management
- **CodeLLDB**: Debugging support

### 3. Configure Rust Analyzer
The workspace is pre-configured to analyze both projects:
- Main BKD library: `./Cargo.toml`
- Tantivy source: `./external/tantivy/Cargo.toml`

## Development Workflows

### Working on BKD Library

#### Build and Test
- **Ctrl+Shift+P** → "Tasks: Run Task" → "BKD: Build Library"
- **Ctrl+Shift+P** → "Tasks: Run Task" → "BKD: Test All"
- **Ctrl+Shift+P** → "Tasks: Run Task" → "BKD: Run KD-Tree Demo"

#### Quick Commands
```bash
# Terminal in BKD folder
cargo build                    # Build library
cargo test                     # Run all tests
cargo run --bin kd-tree       # Run KD-tree demo
cargo test test_spatial_search # Run specific test
```

### Working with Tantivy Source

#### Exploring Code
The workspace gives you full access to Tantivy source with:
- **Go to Definition** (F12) works across both projects
- **Find All References** (Shift+F12)
- **Symbol Search** (Ctrl+T) finds functions/types in both codebases

#### Understanding Integration Points
Key Tantivy files to study:
```
Tantivy Source/
├── src/schema/field_type.rs      # Where we'll add FieldType::Spatial
├── src/indexer/segment_writer.rs # Where spatial indexing happens
├── src/query/                    # Query trait implementations
└── src/directory/                # Storage abstractions we're using
```

#### Tantivy Tasks
- **Ctrl+Shift+P** → "Tasks: Run Task" → "Tantivy: Build"
- **Ctrl+Shift+P** → "Tasks: Run Task" → "Search Tantivy for Pattern"

### Cross-Project Development

#### 1. Prototype in BKD, Integrate in Tantivy
```
BKD Library (Main)/
├── src/bin/kd-tree.rs            # ← Develop NodeLinker abstractions here
└── tests/                        # ← Test spatial algorithms here

Tantivy Source/
├── src/schema/field_type.rs      # ← Add FieldType::Spatial here
├── src/indexer/segment_writer.rs # ← Add spatial indexing logic here
└── src/query/spatial_query.rs    # ← Add spatial queries here (new file)
```

#### 2. Copy Patterns Between Projects
- Develop `NodeLinker` trait in BKD library
- Create `TantivyLinker` implementation in Tantivy source
- Test integration using both workspaces

## Useful VSCode Features

### Multi-Root Search
- **Ctrl+Shift+F**: Search across all workspace folders
- Scope searches: `@BKD` or `@Tantivy` or `@Lucene`

### File Navigation
- **Ctrl+P**: Quick open files from any workspace folder
- Type folder name to scope: "Tantivy field_type.rs"

### Terminal Management
- **Ctrl+Shift+`**: New terminal
- Use dropdown to select working directory (BKD vs Tantivy)

### Debugging
- Set breakpoints in both BKD and Tantivy code
- Use "Debug BKD KD-Tree" launch configuration
- Step through spatial indexing algorithms

## Integration Development Pattern

### Phase 1: Prototype in BKD
```rust
// In BKD Library (Main)/src/bin/kd-tree.rs
trait NodeLinker<P: Point, T> {
    // Develop storage abstraction here
}
```

### Phase 2: Study Tantivy Patterns
```rust
// In Tantivy Source/src/indexer/segment_writer.rs
match field_entry.field_type() {
    FieldType::U64(_) => { /* Study this pattern */ }
    // We'll add FieldType::Spatial here
}
```

### Phase 3: Implement Integration
```rust
// In Tantivy Source/src/schema/field_type.rs
pub enum FieldType {
    // ... existing types
    Spatial(SpatialOptions), // ← Add this
}
```

### Phase 4: Create TantivyLinker
```rust
// In Tantivy Source (new file)
struct TantivyLinker {
    directory: Box<dyn Directory>,
    // Use our NodeLinker trait with Tantivy storage
}

impl NodeLinker<BoundingBox, DocId> for TantivyLinker {
    // Implement using MmapDirectory, OwnedBytes, etc.
}
```

## Research Workflows

### Finding Integration Points
Use the search tasks to find patterns:
1. **Ctrl+Shift+P** → "Tasks: Run Task" → "Search Tantivy for Pattern"
2. Enter: "FieldType" to see how field types are handled
3. Enter: "segment_writer" to see indexing patterns

### Comparing with Lucene
1. **Ctrl+Shift+P** → "Tasks: Run Task" → "Search Lucene for Pattern"
2. Enter: "BKDWriter" to see Lucene's implementation
3. Compare with Tantivy patterns

## Tips for Effective Development

### 1. Use Split Editors
- Open `kd-tree.rs` and `field_type.rs` side-by-side
- Develop abstractions while studying integration points

### 2. Leverage Language Server
- Hover over types to see documentation
- Use "Go to Definition" to understand Tantivy's architecture
- "Find All References" to see usage patterns

### 3. Test Continuously
- Keep BKD tests running: `cargo test --watch`
- Test Tantivy modules: Use "Tantivy: Test Specific Module" task

### 4. Document Integration Points
- Add comments linking BKD abstractions to Tantivy implementations
- Use workspace search to find related patterns

## Next Steps

1. **Open the workspace**: `code bkd-tantivy.code-workspace`
2. **Explore Tantivy structure**: Navigate to `src/schema/field_type.rs`
3. **Study integration patterns**: Look at existing field type implementations
4. **Plan TantivyLinker**: Design how to use `MmapDirectory` with `NodeLinker`

This workspace structure gives you everything needed to develop the BKD library and Tantivy integration in parallel, with full language server support and easy cross-project navigation.
