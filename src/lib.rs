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
use std::simd::{
    Select, cmp::SimdPartialEq, cmp::SimdPartialOrd, f32x16, num::SimdFloat, u16x16, u32x16,
};

/// A buffer for storing arbitrary metadata payloads (e.g., FlatBuffers).
/// It maps a separate region of memory and provides append-only access.
pub struct MmapMetadataBuffer {
    /// The underlying memory mapping.
    pub mmap: MmapMut,
    /// The current write offset within the buffer.
    pub current_offset: usize,
}

impl MmapMetadataBuffer {
    /// Creates a new `MmapMetadataBuffer` from an existing `MmapMut`.
    /// The first 8 bytes of the mapping are assumed to store the current offset.
    pub fn new(mut mmap: MmapMut) -> Self {
        let current_offset = unsafe { *(mmap.as_ptr() as *const u64) as usize };
        if current_offset == 0 {
            // Initialize with 8 bytes for the offset tracker itself
            let initial_offset = 8;
            unsafe { *(mmap.as_mut_ptr() as *mut u64) = initial_offset as u64 };
            Self {
                mmap,
                current_offset: initial_offset,
            }
        } else {
            Self {
                mmap,
                current_offset,
            }
        }
    }

    /// Appends a byte slice to the buffer and returns the starting offset.
    pub fn append(&mut self, data: &[u8]) -> std::result::Result<usize, String> {
        let start_offset = self.current_offset;
        let end_offset = start_offset + data.len();

        if end_offset > self.mmap.len() {
            return Err("MmapMetadataBuffer capacity exceeded".to_string());
        }

        unsafe {
            let dest = self.mmap.as_mut_ptr().add(start_offset);
            std::ptr::copy_nonoverlapping(data.as_ptr(), dest, data.len());
            self.current_offset = end_offset;
            // Persist the new offset
            *(self.mmap.as_mut_ptr() as *mut u64) = end_offset as u64;
        }

        Ok(start_offset)
    }

    /// Returns a slice of the data at the given offset and length.
    pub fn get(&self, offset: usize, len: usize) -> &[u8] {
        &self.mmap[offset..offset + len]
    }
}

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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum GraphEvent {
    NodeAdded { id: u64, type_id: u16 },
    EdgeAdded { src: u32, tgt: u32 },
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct EventPacket {
    pub timestamp: u64,
    pub event: GraphEvent,
}

/// A zero-copy circular ring buffer in Shared Memory to record graph structural events.
pub struct MmapDeltaLog {
    /// The underlying memory mapping.
    pub mmap: MmapMut,
    /// Pointer to the head of the ring buffer (u64).
    pub head_ptr: *mut u64,
    /// Pointer to the tail of the ring buffer (u64).
    pub tail_ptr: *mut u64,
    /// Pointer to the base of the data region.
    pub data_ptr: *mut EventPacket,
    /// Maximum number of packets the buffer can hold.
    pub capacity: usize,
}

impl MmapDeltaLog {
    /// Creates a new `MmapDeltaLog` from an existing `MmapMut`.
    pub fn new(mut mmap: MmapMut) -> Self {
        let base_ptr = mmap.as_mut_ptr();
        // The first 16 bytes store head (8 bytes) and tail (8 bytes)
        let head_ptr = base_ptr as *mut u64;
        let tail_ptr = unsafe { base_ptr.add(8) as *mut u64 };
        let data_start = align_to_64(16);
        let data_ptr = unsafe { base_ptr.add(data_start) as *mut EventPacket };

        let remaining_size = mmap.len() - data_start;
        let capacity = remaining_size / std::mem::size_of::<EventPacket>();

        Self {
            mmap,
            head_ptr,
            tail_ptr,
            data_ptr,
            capacity,
        }
    }

    /// Pushes a new event into the circular buffer.
    pub fn push(&mut self, event: GraphEvent) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let packet = EventPacket { timestamp, event };

        unsafe {
            let tail = *self.tail_ptr;
            let next_tail = (tail + 1) % self.capacity as u64;

            // Write packet at current tail
            self.data_ptr.add(tail as usize).write(packet);

            // Update tail
            *self.tail_ptr = next_tail;

            // If tail catches up to head, advance head (overwrite oldest)
            if next_tail == *self.head_ptr {
                *self.head_ptr = (*self.head_ptr + 1) % self.capacity as u64;
            }
        }
    }
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
    /// The memory-mapped metadata buffer.
    pub metadata: MmapMetadataBuffer,
    /// The memory-mapped delta log for structural events.
    pub delta_log: MmapDeltaLog,
    /// The underlying memory mapping that owns the shared memory region for nodes/edges.
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

        // Metadata is stored in a companion file for now to avoid complexity of shared mapping expansion
        let metadata_path = format!("{}.meta", path);
        let metadata_mmap = init_shared_memory(&metadata_path, 10 * 1024 * 1024)?; // 10MB default
        let metadata = MmapMetadataBuffer::new(metadata_mmap);

        // Delta log for structural events
        let delta_log_path = format!("{}.delta", path);
        let delta_log_mmap = init_shared_memory(&delta_log_path, 1024 * 1024)?; // 1MB default
        let delta_log = MmapDeltaLog::new(delta_log_mmap);

        // SAFETY: We have allocated enough space via `init_shared_memory` for both tables.
        // `nodes_size` ensures that `edges` starts at a valid, aligned boundary.
        let (nodes, edges) = unsafe {
            let nodes = MmapNodeTable::new_from_ptr(base_ptr, node_cap);
            let edges = MmapEdgeTable::new_from_ptr(base_ptr.add(nodes_size), edge_cap);
            (nodes, edges)
        };

        Ok(Self {
            nodes,
            edges,
            metadata,
            delta_log,
            _mmap: mmap,
        })
    }

    /// Adds a new node to the graph.
    pub fn add_node(&mut self, id: u64, type_id: u16, weight: f32) -> usize {
        let idx = self.nodes.add_node(id, type_id, weight);
        self.delta_log.push(GraphEvent::NodeAdded { id, type_id });
        idx
    }

    /// Adds a new edge to the graph.
    pub fn add_edge(&mut self, src: u32, tgt: u32, edge_type: u16, weight: f32) -> usize {
        let idx = self.edges.add_edge(src, tgt, edge_type, weight);
        self.delta_log.push(GraphEvent::EdgeAdded { src, tgt });
        idx
    }

    /// Associates a binary payload (FlatBuffers) with a node.
    pub fn set_node_metadata(
        &mut self,
        node_idx: usize,
        payload: &[u8],
    ) -> std::result::Result<(), String> {
        if node_idx >= self.nodes.count {
            return Err("Invalid node index".to_string());
        }

        let offset = self.metadata.append(payload)?;

        // SAFETY: `ext_offsets_ptr` is valid and `node_idx` is within bounds.
        unsafe {
            self.nodes
                .ext_offsets_ptr
                .add(node_idx)
                .write(offset as u32);
        }

        Ok(())
    }

    /// Retrieves the binary metadata payload for a node.
    pub fn get_node_metadata(&self, node_idx: usize) -> Option<&[u8]> {
        if node_idx >= self.nodes.count {
            return None;
        }

        // SAFETY: `ext_offsets_ptr` is valid and `node_idx` is within bounds.
        let offset = unsafe { *self.nodes.ext_offsets_ptr.add(node_idx) } as usize;

        if offset == 0 {
            return None;
        }

        // FlatBuffers follow the rule that the first 4 bytes at the root
        // point to the actual table data. However, the buffer size itself
        // is needed for safe slicing. For this prototype, we look at the
        // `current_offset` of the metadata buffer to find the boundary.
        // A more robust way would be to store length in the DOD table.
        if offset >= self.metadata.current_offset {
            return None;
        }

        Some(&self.metadata.mmap[offset..self.metadata.current_offset])
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

        let count = self.nodes.count;
        let simd_end = (count / 16) * 16;

        // Level 2 Optimization: Use running best vectors to avoid frequent reductions
        let mut running_best_scores = f32x16::splat(-1.0);
        let mut running_best_indices = u32x16::splat(0);

        // Pre-compute lane indices and increment
        let mut lane_indices =
            u32x16::from_array([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
        let increment = u32x16::splat(16);

        let simd_end_unrolled = (count / 64) * 64;
        let mut i = 0;
        while i < simd_end_unrolled {
            // Process 4 blocks of 16 nodes (64 nodes total) to amortize loop overhead
            for _ in 0..4 {
                let types = u16x16::from_slice(&type_slice[i..i + 16]);
                let mask = types.simd_eq(target_simd);

                if mask.any() {
                    let weights = f32x16::from_slice(&weight_slice[i..i + 16]);
                    let better_weight_mask = weights.simd_gt(running_best_scores);
                    let update_mask = mask & better_weight_mask.cast();

                    if update_mask.any() {
                        running_best_scores = update_mask.select(weights, running_best_scores);
                        running_best_indices =
                            update_mask.select(lane_indices, running_best_indices);
                    }
                }
                lane_indices += increment;
                i += 16;
            }
        }

        // Process remaining blocks between simd_end_unrolled and simd_end
        while i < simd_end {
            let types = u16x16::from_slice(&type_slice[i..i + 16]);
            let mask = types.simd_eq(target_simd);

            if mask.any() {
                let weights = f32x16::from_slice(&weight_slice[i..i + 16]);
                let better_weight_mask = weights.simd_gt(running_best_scores);
                let update_mask = mask & better_weight_mask.cast();

                if update_mask.any() {
                    running_best_scores = update_mask.select(weights, running_best_scores);
                    running_best_indices = update_mask.select(lane_indices, running_best_indices);
                }
            }
            lane_indices += increment;
            i += 16;
        }

        // Final reduction: Only call reduce_max ONCE after the main loop
        let final_max_score = running_best_scores.reduce_max();
        if final_max_score > -1.0 {
            for j in 0..16 {
                if running_best_scores[j] == final_max_score {
                    best_score = final_max_score;
                    best_idx = running_best_indices[j] as usize;
                    break; // Pick the first occurrence
                }
            }
        }

        // Scalar fallback for remainder (unchanged)
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
        let _ = std::fs::remove_file(format!("{}.delta", path));
        let node_cap = 10;
        let edge_cap = 10;

        {
            let mut store = GraphStore::new(path, node_cap, edge_cap).unwrap();
            store.add_node(1, 1, 0.5);
            store.add_edge(0, 1, 1, 0.1);
            assert_eq!(store.nodes.count, 1);
            assert_eq!(store.edges.count, 1);

            // Verify delta log
            unsafe {
                assert_eq!(*store.delta_log.head_ptr, 0);
                assert_eq!(*store.delta_log.tail_ptr, 2);
            }
        }

        {
            let store = GraphStore::new(path, node_cap, edge_cap).unwrap();
            assert_eq!(store.nodes.count, 1);
            assert_eq!(store.edges.count, 1);
            // SAFETY: Testing persistence by reading from a new mapping of the same file.
            unsafe {
                assert_eq!(*store.nodes.ids_ptr, 1);
                assert_eq!(*store.edges.weights_ptr, 0.1);
                assert_eq!(*store.delta_log.head_ptr, 0);
                assert_eq!(*store.delta_log.tail_ptr, 2);
            }
        }
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(format!("{}.delta", path));
    }

    #[test]
    fn test_circular_delta_log() {
        let path = "test_delta.bin";
        let _ = std::fs::remove_file(path);
        let mmap = init_shared_memory(path, 1024).unwrap();
        let mut log = MmapDeltaLog::new(mmap);

        let cap = log.capacity;

        for i in 0..cap + 5 {
            log.push(GraphEvent::NodeAdded {
                id: i as u64,
                type_id: 1,
            });
        }

        unsafe {
            // After cap + 5 pushes, tail should be at 5
            assert_eq!(*log.tail_ptr, 5);
            // And head should have advanced 6 times (from 0 to 6)
            assert_eq!(*log.head_ptr, 6);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_flatbuffers_metadata() {
        use crate::metadata::cntm_graph::{
            NodeMetadata, NodeMetadataArgs, Property, PropertyArgs, Value,
        };
        use flatbuffers::FlatBufferBuilder;

        let path = "test_metadata.bin";
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(format!("{}.meta", path));

        let mut store = GraphStore::new(path, 10, 10).unwrap();
        store.nodes.add_node(1, 1, 0.5);

        // Build FlatBuffers metadata
        let mut fbb = FlatBufferBuilder::new();
        let name = fbb.create_string("AGI-Root");
        let key = fbb.create_string("confidence");
        let val_str = fbb.create_string("0.99");

        let p0 = Property::create(
            &mut fbb,
            &PropertyArgs {
                key: Some(key),
                value_type: Value::FloatValue,
                value: Some(val_str.as_union_value()),
            },
        );

        let properties = fbb.create_vector(&[p0]);

        let metadata_offset = NodeMetadata::create(
            &mut fbb,
            &NodeMetadataArgs {
                name: Some(name),
                properties: Some(properties),
            },
        );
        fbb.finish(metadata_offset, None);
        let payload = fbb.finished_data();

        // Store metadata
        store.set_node_metadata(0, payload).unwrap();

        // Retrieve and verify
        let retrieved_payload = store.get_node_metadata(0).unwrap();
        let metadata = flatbuffers::root::<NodeMetadata>(retrieved_payload).unwrap();
        assert_eq!(metadata.name(), Some("AGI-Root"));

        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(format!("{}.meta", path));
    }
}
