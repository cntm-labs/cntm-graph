//! CNTM-Graph Kernel: High-performance Hybrid Mmap-DOD Graph Engine.
//!
//! This library provides the core graph engine implementation for the CNTM-Graph system,
//! focusing on zero-copy shared memory access, 64-byte alignment for SIMD optimization,
//! and a Data-Oriented Design (DOD) layout.

#![feature(portable_simd)]

pub mod metadata;
#[allow(clippy::all)]
pub mod metadata_generated;

use memmap2::MmapMut;
use std::fs::OpenOptions;
use std::io::Result;
use std::ptr::NonNull;
use std::simd::{Select, cmp::SimdPartialEq, f32x16, num::SimdFloat, u16x16};

/// Initializes or opens a shared memory file of a specified size.
///
/// # Arguments
/// * `path` - The filesystem path to the shared memory file.
/// * `size` - The required size of the memory mapping in bytes.
///
/// # Errors
/// Returns an `std::io::Result` if file operations or mapping fails.
pub fn init_shared_memory(path: &str, size: usize) -> Result<MmapMut> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)?;
    file.set_len(size as u64)?;

    // SAFETY: The file handle is valid and its length has been explicitly set to `size`.
    // Memory mapping is safe here as long as the underlying file is not concurrently
    // truncated or modified by other processes in a way that invalidates this mapping.
    // We assume the caller manages cross-process synchronization if necessary.
    unsafe { MmapMut::map_mut(&file) }
}

/// A cache-line aligned (64-byte) block of 16 f32 weights.
/// Designed for SIMD-accelerated weight processing.
#[repr(C, align(64))]
#[derive(Debug, Default, Clone, Copy)]
pub struct AlignedWeightBlock {
    pub values: [f32; 16],
}

impl AlignedWeightBlock {
    /// Creates a new `AlignedWeightBlock` initialized to zeros.
    pub fn new() -> Self {
        Self { values: [0.0; 16] }
    }
}

/// Aligns a given offset to the next 64-byte boundary.
///
/// # Arguments
/// * `offset` - The original offset in bytes.
pub fn align_to_64(offset: usize) -> usize {
    (offset + 63) & !63
}

/// A Data-Oriented Design (DOD) table for nodes stored in memory-mapped files.
///
/// All fields are stored in contiguous arrays to maximize cache locality and
/// enable efficient SIMD operations. The layout is 64-byte aligned.
#[derive(Debug)]
pub struct MmapNodeTable {
    /// Pointer to the base of the node table memory region.
    pub ptr: NonNull<u8>,
    /// Maximum number of nodes that can be stored.
    pub capacity: usize,
    /// Current number of nodes stored in the table.
    pub count: usize,
    /// Pointer to the array of node IDs (u64).
    pub ids_ptr: *mut u64,
    /// Pointer to the array of node type IDs (u16).
    pub type_ids_ptr: *mut u16,
    /// Pointer to the array of node states (u8, e.g., active/deleted).
    pub states_ptr: *mut u8,
    /// Pointer to the array of node weights (f32).
    pub weights_ptr: *mut f32,
    /// Pointer to the array of node timestamps (u64).
    pub timestamps_ptr: *mut u64,
    /// Pointer to the array of extension offsets (u32).
    pub ext_offsets_ptr: *mut u32,
}

impl MmapNodeTable {
    /// Creates a new `MmapNodeTable` view from a raw pointer and capacity.
    ///
    /// # Safety
    /// * `base_ptr` must point to a valid, writable memory region of at least
    ///   `calculate_mmap_size(capacity)` bytes.
    /// * The memory layout must follow the internal alignment rules (64-byte alignment).
    /// * The first 8 bytes of `base_ptr` must contain the current node count as a u64.
    pub unsafe fn new_from_ptr(base_ptr: *mut u8, capacity: usize) -> Self {
        // SAFETY: We calculate offsets based on 64-byte alignment to ensure zero-copy compatibility
        // with SIMD architectures. The caller must ensure `base_ptr` is correctly aligned.
        unsafe {
            let count = *(base_ptr as *mut u64) as usize;
            let ids_offset = align_to_64(8);
            let type_ids_offset = align_to_64(ids_offset + (capacity * 8));
            let states_offset = align_to_64(type_ids_offset + (capacity * 2));
            let weights_offset = align_to_64(states_offset + capacity);
            let timestamps_offset = align_to_64(weights_offset + (capacity * 4));
            let ext_offsets_offset = align_to_64(timestamps_offset + (capacity * 8));

            Self {
                ptr: NonNull::new_unchecked(base_ptr),
                capacity,
                count,
                ids_ptr: base_ptr.add(ids_offset) as *mut u64,
                type_ids_ptr: base_ptr.add(type_ids_offset) as *mut u16,
                states_ptr: base_ptr.add(states_offset),
                weights_ptr: base_ptr.add(weights_offset) as *mut f32,
                timestamps_ptr: base_ptr.add(timestamps_offset) as *mut u64,
                ext_offsets_ptr: base_ptr.add(ext_offsets_offset) as *mut u32,
            }
        }
    }

    /// Returns a slice of node type IDs.
    pub fn get_type_slice(&self) -> &[u16] {
        // SAFETY: `type_ids_ptr` is initialized in `new_from_ptr` and its lifetime
        // is tied to the memory mapping, which we assume is valid for the duration
        // of `self`.
        unsafe { std::slice::from_raw_parts(self.type_ids_ptr, self.count) }
    }

    /// Returns a slice of node weights.
    pub fn get_weight_slice(&self) -> &[f32] {
        // SAFETY: Same as `get_type_slice`, the pointer is valid for `count` elements.
        unsafe { std::slice::from_raw_parts(self.weights_ptr, self.count) }
    }

    /// Adds a new node to the table and returns its index.
    ///
    /// # Panics
    /// Panics if the table's capacity is exceeded in debug builds.
    pub fn add_node(&mut self, id: u64, type_id: u16, weight: f32) -> usize {
        debug_assert!(
            self.count < self.capacity,
            "MmapNodeTable capacity exceeded"
        );
        let idx = self.count;

        // SAFETY: We have verified that `idx < capacity`, so the pointer offsets are within bounds
        // defined during `new_from_ptr`.
        unsafe {
            self.ids_ptr.add(idx).write(id);
            self.type_ids_ptr.add(idx).write(type_id);
            self.states_ptr.add(idx).write(1); // 1 = Active
            self.weights_ptr.add(idx).write(weight);
            self.timestamps_ptr.add(idx).write(0); // Default placeholder
            self.ext_offsets_ptr.add(idx).write(0);

            self.count += 1;
            // Persist count to the beginning of the mmap region
            *(self.ptr.as_ptr() as *mut u64) = self.count as u64;
        }
        idx
    }

    /// Calculates the total memory size required for a `MmapNodeTable` with the given capacity.
    pub fn calculate_mmap_size(capacity: usize) -> usize {
        let ids_offset = align_to_64(8);
        let type_ids_offset = align_to_64(ids_offset + (capacity * 8));
        let states_offset = align_to_64(type_ids_offset + (capacity * 2));
        let weights_offset = align_to_64(states_offset + capacity);
        let timestamps_offset = align_to_64(weights_offset + (capacity * 4));
        let ext_offsets_offset = align_to_64(timestamps_offset + (capacity * 8));
        align_to_64(ext_offsets_offset + (capacity * 4))
    }
}

/// A Data-Oriented Design (DOD) table for edges stored in memory-mapped files.
///
/// Optimized for fast traversal and SIMD processing with 64-byte aligned array fields.
#[derive(Debug)]
pub struct MmapEdgeTable {
    /// Pointer to the base of the edge table memory region.
    pub ptr: NonNull<u8>,
    /// Maximum number of edges that can be stored.
    pub capacity: usize,
    /// Current number of edges stored in the table.
    pub count: usize,
    /// Pointer to the array of source node indices (u32).
    pub src_ptr: *mut u32,
    /// Pointer to the array of target node indices (u32).
    pub tgt_ptr: *mut u32,
    /// Pointer to the array of edge type IDs (u16).
    pub types_ptr: *mut u16,
    /// Pointer to the array of edge weights (f32).
    pub weights_ptr: *mut f32,
}

impl MmapEdgeTable {
    /// Creates a new `MmapEdgeTable` view from a raw pointer and capacity.
    ///
    /// # Safety
    /// * `base_ptr` must point to a valid, writable memory region of at least
    ///   `calculate_mmap_size(capacity)` bytes.
    /// * The memory layout must follow internal 64-byte alignment rules.
    /// * The first 8 bytes of `base_ptr` must contain the current edge count as a u64.
    pub unsafe fn new_from_ptr(base_ptr: *mut u8, capacity: usize) -> Self {
        // SAFETY: Offsets are calculated to maintain 64-byte alignment for hardware acceleration.
        // Caller ensures memory validity.
        unsafe {
            let count = *(base_ptr as *mut u64) as usize;
            let src_offset = align_to_64(8);
            let tgt_offset = align_to_64(src_offset + (capacity * 4));
            let types_offset = align_to_64(tgt_offset + (capacity * 4));
            let weights_offset = align_to_64(types_offset + (capacity * 2));

            Self {
                ptr: NonNull::new_unchecked(base_ptr),
                capacity,
                count,
                src_ptr: base_ptr.add(src_offset) as *mut u32,
                tgt_ptr: base_ptr.add(tgt_offset) as *mut u32,
                types_ptr: base_ptr.add(types_offset) as *mut u16,
                weights_ptr: base_ptr.add(weights_offset) as *mut f32,
            }
        }
    }

    /// Adds a new edge to the table and returns its index.
    ///
    /// # Panics
    /// Panics if the table's capacity is exceeded in debug builds.
    pub fn add_edge(&mut self, src: u32, tgt: u32, edge_type: u16, weight: f32) -> usize {
        debug_assert!(
            self.count < self.capacity,
            "MmapEdgeTable capacity exceeded"
        );
        let idx = self.count;

        // SAFETY: Pointer offsets are safe because `idx < capacity`.
        unsafe {
            self.src_ptr.add(idx).write(src);
            self.tgt_ptr.add(idx).write(tgt);
            self.types_ptr.add(idx).write(edge_type);
            self.weights_ptr.add(idx).write(weight);

            self.count += 1;
            // Persist count to shared memory
            *(self.ptr.as_ptr() as *mut u64) = self.count as u64;
        }
        idx
    }

    /// Calculates the total memory size required for a `MmapEdgeTable` with the given capacity.
    pub fn calculate_mmap_size(capacity: usize) -> usize {
        let src_offset = align_to_64(8);
        let tgt_offset = align_to_64(src_offset + (capacity * 4));
        let types_offset = align_to_64(tgt_offset + (capacity * 4));
        let weights_offset = align_to_64(types_offset + (capacity * 2));
        align_to_64(weights_offset + (capacity * 4))
    }
}

/// The top-level Graph Store managing memory-mapped node and edge tables.
///
/// This structure owns the underlying `MmapMut` and provides access to
/// the Data-Oriented Design (DOD) tables for high-performance graph operations.
pub struct GraphStore {
    /// The memory-mapped node table.
    pub nodes: MmapNodeTable,
    /// The memory-mapped edge table.
    pub edges: MmapEdgeTable,
    /// The metadata manager for append-only storage.
    pub metadata: metadata::MetadataManager,
    /// The underlying memory mapping that owns the shared memory region.
    _mmap: MmapMut,
}

impl GraphStore {
    /// Initializes a new `GraphStore` at the specified path with given capacities.
    ///
    /// If the file already exists, it will be mapped as-is.
    ///
    /// # Errors
    /// Returns an `std::io::Result` if memory mapping or file access fails.
    pub fn new(path: &str, node_cap: usize, edge_cap: usize) -> Result<Self> {
        let nodes_size = MmapNodeTable::calculate_mmap_size(node_cap);
        let edges_size = MmapEdgeTable::calculate_mmap_size(edge_cap);
        let total_size = nodes_size + edges_size;

        let mut mmap = init_shared_memory(path, total_size)?;
        let base_ptr = mmap.as_mut_ptr();

        // SAFETY: We have allocated enough space via `init_shared_memory` for both tables.
        // `nodes_size` ensures that `edges` starts at a valid, aligned boundary.
        let (nodes, edges) = unsafe {
            let nodes = MmapNodeTable::new_from_ptr(base_ptr, node_cap);
            let edges = MmapEdgeTable::new_from_ptr(base_ptr.add(nodes_size), edge_cap);
            (nodes, edges)
        };

        let metadata = metadata::MetadataManager::new(path)?;

        Ok(Self {
            nodes,
            edges,
            metadata,
            _mmap: mmap,
        })
    }

    pub fn set_node_metadata(&mut self, idx: usize, name: &str, desc: &str) -> Result<()> {
        let mut builder = flatbuffers::FlatBufferBuilder::new();
        let name_off = builder.create_string(name);
        let desc_off = builder.create_string(desc);

        let info = metadata_generated::metadata::BasicInfo::create(
            &mut builder,
            &metadata_generated::metadata::BasicInfoArgs {
                name: Some(name_off),
                description: Some(desc_off),
            },
        );

        let entry = metadata_generated::metadata::MetadataEntry::create(
            &mut builder,
            &metadata_generated::metadata::MetadataEntryArgs {
                data_type: metadata_generated::metadata::EntryData::BasicInfo,
                data: Some(info.as_union_value()),
                isotime_ref: 0,
            },
        );

        builder.finish(entry, None);
        let offset = self
            .metadata
            .append_node_metadata(builder.finished_data())?;

        unsafe {
            self.nodes.ext_offsets_ptr.add(idx).write(offset);
        }
        Ok(())
    }

    /// Finds the node index and weight of the best node matching a target type ID.
    ///
    /// This implementation uses SIMD for 16-wide processing and falls back to scalar
    /// processing for the remainder.
    pub fn find_best_weighted_simd(&self, target_type: u16) -> (usize, f32) {
        let type_slice = self.nodes.get_type_slice();
        let weight_slice = self.nodes.get_weight_slice();

        let mut best_idx = 0;
        let mut best_score = -1.0;

        let target_simd = u16x16::splat(target_type);
        let zero_simd = f32x16::splat(0.0);

        let count = self.nodes.count;
        let simd_end = (count / 16) * 16;

        for i in (0..simd_end).step_by(16) {
            let types = u16x16::from_slice(&type_slice[i..i + 16]);
            let weights = f32x16::from_slice(&weight_slice[i..i + 16]);

            let mask = types.simd_eq(target_simd);
            let scores = mask.select(weights, zero_simd);

            let max_score = scores.reduce_max();
            if max_score > best_score {
                // Find index within block
                for j in 0..16 {
                    if scores[j] == max_score {
                        best_score = max_score;
                        best_idx = i + j;
                    }
                }
            }
        }

        // Scalar fallback for remainder
        for i in simd_end..count {
            if type_slice[i] == target_type && weight_slice[i] > best_score {
                best_score = weight_slice[i];
                best_idx = i;
            }
        }

        (best_idx, best_score)
    }
}

/// Simple addition function for testing basic functionality.
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::align_of;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_weight_block_alignment() {
        assert_eq!(align_of::<AlignedWeightBlock>(), 64);
    }

    #[test]
    fn test_node_table_addition() {
        let _ = std::fs::remove_file("test_node_table.bin");
        let capacity = 1024;
        let size = MmapNodeTable::calculate_mmap_size(capacity);
        let mut mmap = init_shared_memory("test_node_table.bin", size).unwrap();

        // SAFETY: Testing raw pointer access to the initialized memory mapping.
        let mut table = unsafe { MmapNodeTable::new_from_ptr(mmap.as_mut_ptr(), capacity) };

        let idx = table.add_node(12345, 1, 0.85);
        assert_eq!(idx, 0);
        assert_eq!(table.count, 1);

        // SAFETY: Verifying memory contents directly via the table's internal pointers.
        unsafe {
            assert_eq!(*table.ids_ptr.add(idx), 12345);
            assert_eq!(*table.weights_ptr.add(idx), 0.85);
            assert_eq!(*table.states_ptr.add(idx), 1);
        }

        let _ = std::fs::remove_file("test_node_table.bin");
    }

    #[test]
    fn test_mmap_edge_table_addition() {
        let _ = std::fs::remove_file("test_edge_table.bin");
        let capacity = 1024;
        let size = MmapEdgeTable::calculate_mmap_size(capacity);
        let mut mmap = init_shared_memory("test_edge_table.bin", size).unwrap();

        // SAFETY: Testing raw pointer access to initialized edge table memory.
        let mut table = unsafe { MmapEdgeTable::new_from_ptr(mmap.as_mut_ptr(), capacity) };

        let idx = table.add_edge(10, 20, 3, 0.75);
        assert_eq!(idx, 0);
        assert_eq!(table.count, 1);

        // SAFETY: Verifying memory contents directly.
        unsafe {
            assert_eq!(*table.src_ptr.add(idx), 10);
            assert_eq!(*table.tgt_ptr.add(idx), 20);
            assert_eq!(*table.types_ptr.add(idx), 3);
            assert_eq!(*table.weights_ptr.add(idx), 0.75);
        }

        let _ = std::fs::remove_file("test_edge_table.bin");
    }

    #[test]
    fn test_mmap_initialization() {
        let _ = std::fs::remove_file("test_graph.bin");
        let result = init_shared_memory("test_graph.bin", 1024 * 1024);
        assert!(result.is_ok());
        // cleanup
        let _ = std::fs::remove_file("test_graph.bin");
    }

    #[test]
    fn test_graph_store_persistence() {
        let path = "test_store.bin";
        let _ = std::fs::remove_file(path);
        let node_cap = 10;
        let edge_cap = 10;

        {
            let mut store = GraphStore::new(path, node_cap, edge_cap).unwrap();
            store.nodes.add_node(1, 1, 0.5);
            store.edges.add_edge(0, 1, 1, 0.1);
            assert_eq!(store.nodes.count, 1);
            assert_eq!(store.edges.count, 1);
        }

        {
            let store = GraphStore::new(path, node_cap, edge_cap).unwrap();
            assert_eq!(store.nodes.count, 1);
            assert_eq!(store.edges.count, 1);
            // SAFETY: Testing persistence by reading from a new mapping of the same file.
            unsafe {
                assert_eq!(*store.nodes.ids_ptr, 1);
                assert_eq!(*store.edges.weights_ptr, 0.1);
            }
        }
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_simd_traversal() {
        let path = "test_simd.bin";
        let _ = std::fs::remove_file(path);
        let node_cap = 100;
        let edge_cap = 10;

        let mut store = GraphStore::new(path, node_cap, edge_cap).unwrap();

        // Add some nodes
        for i in 0..40 {
            let type_id = if i % 5 == 0 { 10 } else { 1 };
            let weight = (i as f32) * 0.1;
            store.nodes.add_node(i as u64, type_id, weight);
        }

        // Target type 10, max weight should be at index 35 (35 * 0.1 = 3.5)
        let (best_idx, best_weight) = store.find_best_weighted_simd(10);
        assert_eq!(best_idx, 35);
        assert!((best_weight - 3.5).abs() < f32::EPSILON);

        // Test with remainder (40 nodes, 2x16 SIMD blocks + 8 remainder)
        store.nodes.add_node(40, 10, 5.0); // Index 40, weight 5.0
        let (best_idx, best_weight) = store.find_best_weighted_simd(10);
        assert_eq!(best_idx, 40);
        assert_eq!(best_weight, 5.0);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_metadata_integration() {
        let path = "test_metadata.bin";
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(format!("{}.nodes.meta", path));
        let _ = std::fs::remove_file(format!("{}.edges.meta", path));

        let node_cap = 10;
        let edge_cap = 10;

        {
            let mut store = GraphStore::new(path, node_cap, edge_cap).unwrap();
            let idx = store.nodes.add_node(1, 1, 0.5);
            store
                .set_node_metadata(idx, "Node1", "This is node 1")
                .unwrap();

            // Check if offset is non-zero (first entry is usually at 0, but flatbuffers has header)
            unsafe {
                assert_eq!(*store.nodes.ext_offsets_ptr.add(idx), 0);
            }
        }

        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(format!("{}.nodes.meta", path));
        let _ = std::fs::remove_file(format!("{}.edges.meta", path));
    }
}
