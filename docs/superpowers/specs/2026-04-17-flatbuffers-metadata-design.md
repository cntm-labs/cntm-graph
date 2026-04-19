# Hybrid FlatBuffers Metadata Design (Cold Path)

- **Date:** 2026-04-17
- **Status:** Approved
- **Topic:** Zero-copy Metadata Management for AGI Graph Engine

## 1. Objective
Implement a high-performance, asynchronous metadata system ("Cold Path") using **FlatBuffers**. This system allows the `cntm-graph` to store rich information (names, descriptions, provenance) without impacting the sub-nanosecond latency of the "Hot Path" (traversal and weight calculation).

## 2. Architectural Principles
- **Separation of Concerns:** Keep Hot Path data (mmap pointers) and Cold Path data (FlatBuffers) in separate physical files to maximize data locality.
- **Append-only Evolution:** Metadata is updated by appending new entries to the end of the file, preserving a chronological history that integrates with **Isotime**.
- **Zero-copy FFI:** Ensure both Rust and Mojo can read metadata directly from shared memory without deserialization overhead.

## 3. Data Layout

### 3.1 Hot/Cold Bridge
Each node and edge in the Hot Path contains an `ext_offset` (u32). 
- `0`: No metadata associated.
- `>0`: Byte offset from the start of the corresponding `.meta` file.

### 3.2 FlatBuffers Schema (`metadata.fbs`)
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
  isotime_ref: ulong; // Reference to Isotime temporal layer
}

root_type MetadataEntry;
```

## 4. Components

### 4.1 Rust Metadata Manager (`src/metadata.rs`)
- Manages file handles for `nodes.meta` and `edges.meta`.
- Provides an `append_node_metadata` function that returns the new offset.
- Integrates with `GraphStore` to update Hot Path offsets.

### 4.2 Mojo Zero-copy Reader (`src/main.mojo`)
- Maps `.meta` files into memory.
- Uses `UnsafePointer` and FlatBuffers memory layout to read fields without copying.

## 5. Workflow: Metadata Update
1. **Request:** A node's metadata needs to change (e.g., "Self-healing" update).
2. **Serialize:** Rust serializes the new `MetadataEntry` using FlatBuffers.
3. **Append:** The serialized buffer is appended to `nodes.meta`.
4. **Link:** The `ext_offset` in the Hot Path `MmapNodeTable` is updated to the new position.
5. **Sync:** A delta is sent to **Isotime** to record the temporal event.

## 6. Success Criteria
- **Traversal Performance:** 0% degradation in Hot Path traversal speed.
- **Memory Efficiency:** Cold Path data is only loaded into the kernel page cache when accessed.
- **Isotime Compatibility:** Full audit trail of metadata changes available via offset history.
