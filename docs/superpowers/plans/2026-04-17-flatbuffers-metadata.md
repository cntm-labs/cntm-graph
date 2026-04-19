# Hybrid FlatBuffers Metadata Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement an append-only FlatBuffers metadata storage for nodes and edges, linked by offsets in the Hot Path.

**Architecture:** Use `flatbuffers` for zero-copy serialization. Rust handles appending to `.meta` files, while both Rust and Mojo read via mmap.

**Tech Stack:** Rust, `flatbuffers`, `memmap2`, Mojo.

---

### Task 1: Define FlatBuffers Schema

**Files:**
- Create: `src/metadata.fbs`

- [ ] **Step 1: Create the schema file**

```flatbuffers
namespace Metadata;

table BasicInfo {
  name: string;
  description: string;
}

table SemanticInfo {
  external_id: string;
  provenance: string;
  confidence: float;
}

union EntryData { BasicInfo, SemanticInfo }

table MetadataEntry {
  data: EntryData;
  isotime_ref: ulong;
}

root_type MetadataEntry;
```

- [ ] **Step 2: Generate Rust code from schema**

Run: `flatc --rust -o src/ src/metadata.fbs`

- [ ] **Step 3: Commit**

```bash
git add src/metadata.fbs src/metadata_generated.rs
git commit -m "feat: add FlatBuffers schema and generated Rust code"
```

---

### Task 2: Implement Metadata Manager

**Files:**
- Create: `src/metadata.rs`
- Modify: `src/lib.rs` (to export metadata module)

- [ ] **Step 1: Implement `MetadataManager` for append-only writes**

```rust
use std::fs::{OpenOptions, File};
use std::io::{Write, Seek, SeekFrom};
use std::path::Path;

pub struct MetadataManager {
    node_file: File,
    edge_file: File,
}

impl MetadataManager {
    pub fn new(base_path: &str) -> std::io::Result<Self> {
        let node_path = format!("{}.nodes.meta", base_path);
        let edge_path = format!("{}.edges.meta", base_path);
        
        let node_file = OpenOptions::new().create(true).append(true).read(true).open(node_path)?;
        let edge_file = OpenOptions::new().create(true).append(true).read(true).open(edge_path)?;
        
        Ok(Self { node_file, edge_file })
    }

    pub fn append_node_metadata(&mut self, data: &[u8]) -> std::io::Result<u32> {
        let offset = self.node_file.seek(SeekFrom::End(0))? as u32;
        self.node_file.write_all(data)?;
        Ok(offset)
    }

    pub fn append_edge_metadata(&mut self, data: &[u8]) -> std::io::Result<u32> {
        let offset = self.edge_file.seek(SeekFrom::End(0))? as u32;
        self.edge_file.write_all(data)?;
        Ok(offset)
    }
}
```

- [ ] **Step 2: Export module in `src/lib.rs`**

Add `pub mod metadata;` and `pub mod metadata_generated;` to `src/lib.rs`.

- [ ] **Step 3: Commit**

```bash
git add src/metadata.rs src/lib.rs
git commit -m "feat: implement append-only MetadataManager"
```

---

### Task 3: Integrate with GraphStore

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Update `GraphStore` to include `MetadataManager`**

- [ ] **Step 2: Add `set_node_metadata` method**

```rust
impl GraphStore {
    pub fn set_node_metadata(&mut self, idx: usize, name: &str, desc: &str) -> std::io::Result<()> {
        let mut builder = flatbuffers::FlatBufferBuilder::with_capacity(1024);
        let name_off = builder.create_string(name);
        let desc_off = builder.create_string(desc);
        
        let info = metadata_generated::Metadata::BasicInfo::create(&mut builder, &metadata_generated::Metadata::BasicInfoArgs {
            name: Some(name_off),
            description: Some(desc_off),
        });
        
        let entry = metadata_generated::Metadata::MetadataEntry::create(&mut builder, &metadata_generated::Metadata::MetadataEntryArgs {
            data_type: metadata_generated::Metadata::EntryData::BasicInfo,
            data: Some(info.as_union_value()),
            isotime_ref: 0,
        });
        
        builder.finish(entry, None);
        let offset = self.metadata.append_node_metadata(builder.finished_data())?;
        
        unsafe {
            self.nodes.ext_offsets_ptr.add(idx).write(offset);
        }
        Ok(())
    }
}
```

- [ ] **Step 3: Write integration test**

Verify metadata can be written and the offset is correctly stored in the Hot Path.

- [ ] **Step 4: Commit**

```bash
git add src/lib.rs
git commit -m "feat: integrate MetadataManager into GraphStore"
```

---

### Task 4: Mojo Implementation

**Files:**
- Modify: `src/main.mojo`

- [ ] **Step 1: Add metadata file mapping to Mojo**

- [ ] **Step 2: Implement basic metadata reading in Mojo**

```mojo
# In src/main.mojo
# Add logic to map nodes.meta and read strings at given offsets
```

- [ ] **Step 3: Commit**

```bash
git add src/main.mojo
git commit -m "feat: implement Mojo metadata reading"
```
