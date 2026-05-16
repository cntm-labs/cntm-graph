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

/// Canary token used for guard segments to detect memory corruption.
pub const CANARY_PATTERN: u64 = 0xDEADBEEFCAFEBABE;
/// Size of the guard segment in bytes.
pub const GUARD_SIZE: usize = 64;

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
    MetadataUpdated { node_idx: u32 },
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
/// enable efficient SIMD operations. The layout is 64-byte aligned and protected
/// by canary guard segments.
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
    /// Pointer to the array of extension lengths (u32).
    pub ext_lengths_ptr: *mut u32,
    /// Pointer to the dirty bitmask (u64 words, each covering 64 nodes).
    pub dirty_mask_ptr: *mut u64,
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
        // with SIMD architectures. Guard segments are inserted between arrays.
        unsafe {
            let count = *(base_ptr as *mut u64) as usize;
            let ids_offset = align_to_64(8);
            let type_ids_offset =
                align_to_64(align_to_64(ids_offset + (capacity * 8)) + GUARD_SIZE);
            let states_offset =
                align_to_64(align_to_64(type_ids_offset + (capacity * 2)) + GUARD_SIZE);
            let weights_offset = align_to_64(align_to_64(states_offset + capacity) + GUARD_SIZE);
            let timestamps_offset =
                align_to_64(align_to_64(weights_offset + (capacity * 4)) + GUARD_SIZE);
            let ext_offsets_offset =
                align_to_64(align_to_64(timestamps_offset + (capacity * 8)) + GUARD_SIZE);
            let ext_lengths_offset =
                align_to_64(align_to_64(ext_offsets_offset + (capacity * 4)) + GUARD_SIZE);
            let dirty_mask_offset =
                align_to_64(align_to_64(ext_lengths_offset + (capacity * 4)) + GUARD_SIZE);

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
                ext_lengths_ptr: base_ptr.add(ext_lengths_offset) as *mut u32,
                dirty_mask_ptr: base_ptr.add(dirty_mask_offset) as *mut u64,
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

    /// Marks a node as dirty in the bitmask.
    pub fn mark_dirty(&mut self, idx: usize) {
        let word_idx = idx / 64;
        let bit_idx = idx % 64;
        unsafe {
            let word_ptr = self.dirty_mask_ptr.add(word_idx);
            *word_ptr |= 1 << bit_idx;
        }
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
            self.ext_lengths_ptr.add(idx).write(0);

            self.count += 1;
            // Persist count to the beginning of the mmap region
            *(self.ptr.as_ptr() as *mut u64) = self.count as u64;
        }
        self.mark_dirty(idx);
        idx
    }

    /// Calculates the total memory size required for a `MmapNodeTable` with the given capacity.
    pub fn calculate_mmap_size(capacity: usize) -> usize {
        let ids_offset = align_to_64(8);
        let type_ids_offset = align_to_64(align_to_64(ids_offset + (capacity * 8)) + GUARD_SIZE);
        let states_offset = align_to_64(align_to_64(type_ids_offset + (capacity * 2)) + GUARD_SIZE);
        let weights_offset = align_to_64(align_to_64(states_offset + capacity) + GUARD_SIZE);
        let timestamps_offset =
            align_to_64(align_to_64(weights_offset + (capacity * 4)) + GUARD_SIZE);
        let ext_offsets_offset =
            align_to_64(align_to_64(timestamps_offset + (capacity * 8)) + GUARD_SIZE);
        let ext_lengths_offset =
            align_to_64(align_to_64(ext_offsets_offset + (capacity * 4)) + GUARD_SIZE);
        let dirty_mask_offset =
            align_to_64(align_to_64(ext_lengths_offset + (capacity * 4)) + GUARD_SIZE);
        // Each u64 covers 64 nodes.
        let bitmask_words = capacity.div_ceil(64);
        align_to_64(align_to_64(dirty_mask_offset + (bitmask_words * 8)) + GUARD_SIZE)
    }

    /// Returns a list of all guard segment base pointers for the node table.
    pub fn get_guard_pointers(&self) -> Vec<*mut u64> {
        let capacity = self.capacity;
        let base_ptr = self.ptr.as_ptr();
        let mut guards = Vec::new();
        unsafe {
            let ids_offset = align_to_64(8);
            guards.push(base_ptr.add(align_to_64(ids_offset + (capacity * 8))) as *mut u64);

            let type_ids_offset =
                align_to_64(align_to_64(ids_offset + (capacity * 8)) + GUARD_SIZE);
            guards.push(base_ptr.add(align_to_64(type_ids_offset + (capacity * 2))) as *mut u64);

            let states_offset =
                align_to_64(align_to_64(type_ids_offset + (capacity * 2)) + GUARD_SIZE);
            guards.push(base_ptr.add(align_to_64(states_offset + capacity)) as *mut u64);

            let weights_offset = align_to_64(align_to_64(states_offset + capacity) + GUARD_SIZE);
            guards.push(base_ptr.add(align_to_64(weights_offset + (capacity * 4))) as *mut u64);

            let timestamps_offset =
                align_to_64(align_to_64(weights_offset + (capacity * 4)) + GUARD_SIZE);
            guards.push(base_ptr.add(align_to_64(timestamps_offset + (capacity * 8))) as *mut u64);

            let ext_offsets_offset =
                align_to_64(align_to_64(timestamps_offset + (capacity * 8)) + GUARD_SIZE);
            guards.push(base_ptr.add(align_to_64(ext_offsets_offset + (capacity * 4))) as *mut u64);

            let ext_lengths_offset =
                align_to_64(align_to_64(ext_offsets_offset + (capacity * 4)) + GUARD_SIZE);
            guards.push(base_ptr.add(align_to_64(ext_lengths_offset + (capacity * 4))) as *mut u64);

            let dirty_mask_offset =
                align_to_64(align_to_64(ext_lengths_offset + (capacity * 4)) + GUARD_SIZE);
            let bitmask_words = capacity.div_ceil(64);
            guards.push(
                base_ptr.add(align_to_64(dirty_mask_offset + (bitmask_words * 8))) as *mut u64,
            );
        }
        guards
    }
}

/// A Data-Oriented Design (DOD) table for edges stored in memory-mapped files.
///
/// Optimized for fast traversal and SIMD processing with 64-byte aligned array fields
/// and canary guard protection.
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
        // SAFETY: Offsets are calculated to maintain 64-byte alignment and include guards.
        unsafe {
            let count = *(base_ptr as *mut u64) as usize;
            let src_offset = align_to_64(8);
            let tgt_offset = align_to_64(align_to_64(src_offset + (capacity * 4)) + GUARD_SIZE);
            let types_offset = align_to_64(align_to_64(tgt_offset + (capacity * 4)) + GUARD_SIZE);
            let weights_offset =
                align_to_64(align_to_64(types_offset + (capacity * 2)) + GUARD_SIZE);

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
        let tgt_offset = align_to_64(align_to_64(src_offset + (capacity * 4)) + GUARD_SIZE);
        let types_offset = align_to_64(align_to_64(tgt_offset + (capacity * 4)) + GUARD_SIZE);
        let weights_offset = align_to_64(align_to_64(types_offset + (capacity * 2)) + GUARD_SIZE);
        align_to_64(align_to_64(weights_offset + (capacity * 4)) + GUARD_SIZE)
    }

    /// Returns a list of all guard segment base pointers for the edge table.
    pub fn get_guard_pointers(&self) -> Vec<*mut u64> {
        let capacity = self.capacity;
        let base_ptr = self.ptr.as_ptr();
        let mut guards = Vec::new();
        unsafe {
            let src_offset = align_to_64(8);
            guards.push(base_ptr.add(align_to_64(src_offset + (capacity * 4))) as *mut u64);

            let tgt_offset = align_to_64(align_to_64(src_offset + (capacity * 4)) + GUARD_SIZE);
            guards.push(base_ptr.add(align_to_64(tgt_offset + (capacity * 4))) as *mut u64);

            let types_offset = align_to_64(align_to_64(tgt_offset + (capacity * 4)) + GUARD_SIZE);
            guards.push(base_ptr.add(align_to_64(types_offset + (capacity * 2))) as *mut u64);

            let weights_offset =
                align_to_64(align_to_64(types_offset + (capacity * 2)) + GUARD_SIZE);
            guards.push(base_ptr.add(align_to_64(weights_offset + (capacity * 4))) as *mut u64);
        }
        guards
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

/// Report containing memory fragmentation metrics for the metadata arena.
pub struct FragmentationReport {
    /// Total size of the allocated memory-mapped region.
    pub total_arena_size: usize,
    /// Current write offset in the buffer (including the 8-byte header).
    pub current_offset: usize,
    /// Sum of all active metadata payload lengths.
    pub alive_bytes: usize,
    /// Bytes that are no longer referenced by any active node (fragmented).
    pub dead_bytes: usize,
    /// Ratio of dead bytes to the total occupied space (excluding header).
    pub fragmentation_ratio: f32,
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

        let file_existed = std::path::Path::new(path).exists();
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
            // If file is new, initialize count to 0. If it existed, keep existing data.
            if !file_existed {
                *(base_ptr as *mut u64) = 0;
                *(base_ptr.add(nodes_size) as *mut u64) = 0;
            }

            let nodes = MmapNodeTable::new_from_ptr(base_ptr, node_cap);
            let edges = MmapEdgeTable::new_from_ptr(base_ptr.add(nodes_size), edge_cap);
            (nodes, edges)
        };

        let mut store = Self {
            nodes,
            edges,
            metadata,
            delta_log,
            _mmap: mmap,
        };

        if !file_existed {
            store.initialize_guards();
        }

        Ok(store)
    }

    /// Writes the `CANARY_PATTERN` into all guard segments.
    pub fn initialize_guards(&mut self) {
        let mut guards = self.nodes.get_guard_pointers();
        guards.extend(self.edges.get_guard_pointers());

        for guard_ptr in guards {
            unsafe {
                // Fill the 64-byte guard with the 8-byte canary pattern
                for i in 0..(GUARD_SIZE / 8) {
                    guard_ptr.add(i).write(CANARY_PATTERN);
                }
            }
        }
    }

    /// Verifies that all canary patterns in guard segments are still intact.
    pub fn verify_canaries(&self) -> bool {
        let mut guards = self.nodes.get_guard_pointers();
        guards.extend(self.edges.get_guard_pointers());

        for guard_ptr in guards {
            unsafe {
                for i in 0..(GUARD_SIZE / 8) {
                    if *guard_ptr.add(i) != CANARY_PATTERN {
                        return false;
                    }
                }
            }
        }
        true
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
            self.nodes
                .ext_lengths_ptr
                .add(node_idx)
                .write(payload.len() as u32);
        }

        self.nodes.mark_dirty(node_idx);
        self.delta_log.push(GraphEvent::MetadataUpdated {
            node_idx: node_idx as u32,
        });

        Ok(())
    }

    /// Retrieves the binary metadata payload for a node.
    pub fn get_node_metadata(&self, node_idx: usize) -> Option<&[u8]> {
        if node_idx >= self.nodes.count {
            return None;
        }

        // SAFETY: `ext_offsets_ptr` and `ext_lengths_ptr` are valid and `node_idx` is within bounds.
        let (offset, len) = unsafe {
            (
                *self.nodes.ext_offsets_ptr.add(node_idx) as usize,
                *self.nodes.ext_lengths_ptr.add(node_idx) as usize,
            )
        };

        if offset == 0 || len == 0 {
            return None;
        }

        if offset + len > self.metadata.mmap.len() {
            return None;
        }

        Some(&self.metadata.mmap[offset..offset + len])
    }

    /// Performs an analysis of the metadata arena to determine fragmentation levels.
    pub fn analyze_fragmentation(&self) -> FragmentationReport {
        let total_arena_size = self.metadata.mmap.len();
        let current_offset = self.metadata.current_offset;

        let mut alive_bytes = 0;
        for i in 0..self.nodes.count {
            unsafe {
                let offset = *self.nodes.ext_offsets_ptr.add(i);
                if offset > 0 {
                    alive_bytes += *self.nodes.ext_lengths_ptr.add(i) as usize;
                }
            }
        }

        // Subtracting 8 bytes for the header that tracks the current offset
        let dead_bytes = if current_offset > 8 {
            current_offset - 8 - alive_bytes
        } else {
            0
        };

        let fragmentation_ratio = if current_offset > 8 {
            dead_bytes as f32 / (current_offset - 8) as f32
        } else {
            0.0
        };

        FragmentationReport {
            total_arena_size,
            current_offset,
            alive_bytes,
            dead_bytes,
            fragmentation_ratio,
        }
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
    fn test_dirty_bitmask() {
        let _ = std::fs::remove_file("test_dirty_mask.bin");
        let capacity = 128; // 2 words (u64)
        let size = MmapNodeTable::calculate_mmap_size(capacity);
        let mut mmap = init_shared_memory("test_dirty_mask.bin", size).unwrap();

        let mut table = unsafe { MmapNodeTable::new_from_ptr(mmap.as_mut_ptr(), capacity) };

        // Initially zero
        unsafe {
            assert_eq!(*table.dirty_mask_ptr, 0);
            assert_eq!(*table.dirty_mask_ptr.add(1), 0);
        }

        table.add_node(1, 1, 0.5); // idx 0
        unsafe {
            assert_eq!(*table.dirty_mask_ptr, 1);
        }

        table.add_node(2, 1, 0.5); // idx 1
        unsafe {
            assert_eq!(*table.dirty_mask_ptr, 3);
        }

        // Test bit 64 (start of second word)
        for i in 2..64 {
            table.add_node(i + 1, 1, 0.5);
        }
        assert_eq!(table.count, 64);

        table.add_node(65, 1, 0.5); // idx 64
        unsafe {
            assert_eq!(*table.dirty_mask_ptr.add(1), 1);
        }

        let _ = std::fs::remove_file("test_dirty_mask.bin");
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
        let _ = std::fs::remove_file(format!("{}.delta", path));

        let mut store = GraphStore::new(path, 10, 10).unwrap();
        store.add_node(1, 1, 0.5);

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

        // Verify delta log for MetadataUpdated event
        unsafe {
            // Index 0 was NodeAdded, Index 1 is MetadataUpdated
            assert_eq!(*store.delta_log.tail_ptr, 2);
            let packet = *store.delta_log.data_ptr.add(1);
            match packet.event {
                GraphEvent::MetadataUpdated { node_idx } => {
                    assert_eq!(node_idx, 0);
                }
                _ => panic!("Expected MetadataUpdated event"),
            }
        }

        // Retrieve and verify
        let retrieved_payload = store.get_node_metadata(0).unwrap();
        let metadata = flatbuffers::root::<NodeMetadata>(retrieved_payload).unwrap();
        assert_eq!(metadata.name(), Some("AGI-Root"));

        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(format!("{}.meta", path));
    }

    #[test]
    fn test_isotime_handshake() {
        let path = "test_handshake.bin";
        let delta_path = format!("{}.delta", path);
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(format!("{}.meta", path));
        let _ = std::fs::remove_file(&delta_path);

        {
            let mut store = GraphStore::new(path, 10, 10).unwrap();
            store.add_node(101, 1, 0.9);
            store.add_node(102, 1, 0.8);
            store.add_node(103, 2, 0.7);

            assert_eq!(store.nodes.count, 3);

            // Directly check delta log
            unsafe {
                assert_eq!(*store.delta_log.head_ptr, 0);
                assert_eq!(*store.delta_log.tail_ptr, 3);

                // Verify the 3 NodeAdded events
                for i in 0..3 {
                    let packet = *store.delta_log.data_ptr.add(i);
                    match packet.event {
                        GraphEvent::NodeAdded { id, .. } => {
                            assert_eq!(id, (101 + i) as u64);
                        }
                        _ => panic!("Expected NodeAdded event"),
                    }
                }
            }
        }

        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(format!("{}.meta", path));
        let _ = std::fs::remove_file(&delta_path);
    }

    #[test]
    fn test_fragmentation_analysis() {
        let path = "test_frag.bin";
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(format!("{}.meta", path));
        let _ = std::fs::remove_file(format!("{}.delta", path));

        {
            let mut store = GraphStore::new(path, 10, 10).unwrap();
            store.add_node(1, 1, 0.5);
            store.add_node(2, 1, 0.6);

            // Initial report (no metadata)
            let report = store.analyze_fragmentation();
            assert_eq!(report.current_offset, 8);
            assert_eq!(report.alive_bytes, 0);
            assert_eq!(report.dead_bytes, 0);
            assert_eq!(report.fragmentation_ratio, 0.0);

            // Set metadata for node 0
            store.set_node_metadata(0, &[1, 2, 3, 4]).unwrap();
            let report = store.analyze_fragmentation();
            assert_eq!(report.current_offset, 12); // 8 + 4
            assert_eq!(report.alive_bytes, 4);
            assert_eq!(report.dead_bytes, 0);
            assert_eq!(report.fragmentation_ratio, 0.0);

            // Set metadata for node 1
            store.set_node_metadata(1, &[5, 6, 7]).unwrap();
            let report = store.analyze_fragmentation();
            assert_eq!(report.current_offset, 15); // 12 + 3
            assert_eq!(report.alive_bytes, 7);
            assert_eq!(report.dead_bytes, 0);
            assert_eq!(report.fragmentation_ratio, 0.0);

            // Overwrite node 0 metadata (simulating fragmentation)
            // Note: Current implementation of set_node_metadata ALWAYS appends.
            store.set_node_metadata(0, &[8, 9]).unwrap();
            let report = store.analyze_fragmentation();
            // current_offset: 15 + 2 = 17
            // alive_bytes: node 0 len (2) + node 1 len (3) = 5
            // dead_bytes: (17 - 8) - 5 = 9 - 5 = 4
            // fragmentation_ratio: 4 / 9 = 0.444...
            assert_eq!(report.current_offset, 17);
            assert_eq!(report.alive_bytes, 5);
            assert_eq!(report.dead_bytes, 4);
            assert!(report.fragmentation_ratio > 0.44 && report.fragmentation_ratio < 0.45);
        }

        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(format!("{}.meta", path));
        let _ = std::fs::remove_file(format!("{}.delta", path));
    }

    #[test]
    fn test_canary_guards() {
        let path = "test_canary.bin";
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(format!("{}.meta", path));
        let _ = std::fs::remove_file(format!("{}.delta", path));

        {
            let store = GraphStore::new(path, 10, 10).unwrap();
            // Should be valid initially
            assert!(store.verify_canaries());

            // Manually corrupt a canary
            let guards = store.nodes.get_guard_pointers();
            unsafe {
                guards[0].write(0xBAD0000000000BAD);
            }

            // Should now fail verification
            assert!(!store.verify_canaries());
        }

        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(format!("{}.meta", path));
        let _ = std::fs::remove_file(format!("{}.delta", path));
    }
}
