# Tantivy Fork Setup Guide

## TL;DR: Fork Structure

```
your-workspace/
├── bkd/                    # This repository
│   ├── src/               # Your spatial indexing library
│   ├── external/tantivy/  # Original Tantivy (reference only)
│   └── bkd-tantivy.code-workspace
└── tantivy-fork/          # Your Tantivy fork (integration work)
    ├── src/
    ├── Cargo.toml
    └── .git/
```

## Setup Commands

```bash
# 1. Navigate to workspace root (parent of bkd/)
cd ..

# 2. Fork Tantivy on GitHub first, then:
git clone https://github.com/YOUR_USERNAME/tantivy.git tantivy-fork
cd tantivy-fork

# 3. Add upstream for syncing
git remote add upstream https://github.com/quickwit-oss/tantivy.git
git fetch upstream
git checkout main
git rebase upstream/main

# 4. Open the workspace
cd ../bkd
code bkd-tantivy.code-workspace
```

## Why This Structure?

### ✅ **Advantages:**
- **Clean separation**: BKD library development vs Tantivy integration
- **Version control**: Your fork has its own git history
- **Upstream sync**: Easy to merge upstream Tantivy changes
- **VSCode integration**: Full language server support for both projects
- **Comparison**: Easy to diff your changes against original Tantivy

### ❌ **Don't Put Fork Inside BKD:**
- Makes BKD repo huge and cluttered
- Git submodules are complex and fragile
- Hard to manage separate development cycles
- VSCode workspace becomes confusing

## Development Workflow

### **Where to Work:**
- **BKD Library (Main)**: Develop spatial algorithms (`src/spatial.rs`, `src/storage.rs`, etc.)
- **Tantivy Fork (Integration)**: Add spatial features to Tantivy
- **Tantivy Source (Reference)**: Study patterns, don't modify
- **Lucene Reference**: Algorithm reference, don't modify

### **Integration Pattern:**
1. **Develop** spatial algorithms in BKD library
2. **Test** with `cargo test --bin kd-tree` (15 tests)
3. **Copy** working code to Tantivy fork
4. **Integrate** with Tantivy's field types and queries
5. **Test** integration in Tantivy

## VSCode Workspace Features

Once set up, you get:

### **Cross-Project Navigation:**
- **F12 (Go to Definition)**: Jump between BKD and Tantivy code
- **Shift+F12 (Find References)**: See usage across both projects
- **Ctrl+T (Symbol Search)**: Find functions in both codebases

### **Integrated Tasks:**
- `Tantivy Fork: Build with Spatial`
- `Tantivy Fork: Test Spatial Integration`
- `Tantivy Fork: Sync with Upstream`

### **Side-by-Side Development:**
- Edit your spatial algorithms in BKD
- See Tantivy integration points in fork
- Reference original Tantivy patterns
- Compare your changes with original

## File Structure After Setup

```
VSCode Workspace Folders:
├── BKD Library (Main)           # Your spatial library
│   ├── src/spatial.rs          # Point traits, BoundingBox
│   ├── src/storage.rs          # NodeLinker abstraction
│   └── src/search.rs           # spatial_search algorithm
├── Tantivy Fork (Integration)   # Your integration work
│   ├── src/schema/field_type.rs     # Add FieldType::Spatial
│   ├── src/indexer/segment_writer.rs # Add spatial indexing
│   ├── src/query/spatial_query.rs    # New spatial queries
│   └── src/spatial/                  # Copy BKD algorithms here
├── Tantivy Source (Reference)   # Original for comparison
│   └── src/                    # Study patterns here
└── Lucene Reference            # Algorithm reference
    └── lucene/core/src/java/org/apache/lucene/util/bkd/
```

## Next Steps

1. **Set up the fork** using the commands above
2. **Open the workspace**: `code bkd-tantivy.code-workspace`
3. **Verify setup**: All four folders should appear in VSCode
4. **Start integration**: Begin with `FieldType::Spatial` in your fork

This structure gives you the perfect environment for developing spatial indexing while maintaining clean separation between your library and Tantivy integration work!
