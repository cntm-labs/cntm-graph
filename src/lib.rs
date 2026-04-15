use memmap2::MmapMut;
use std::fs::OpenOptions;
use std::io::Result;
use std::ptr::NonNull;

pub fn init_shared_memory(path: &str, size: usize) -> Result<MmapMut> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    file.set_len(size as u64)?;
    // SAFETY: The file handle is valid and its length has been explicitly set.
    // Memory mapping is safe as long as the underlying file is not concurrently
    // truncated or modified in an incompatible way by other processes.
    unsafe { MmapMut::map_mut(&file) }
}

#[repr(C, align(64))]
#[derive(Debug, Default, Clone, Copy)]
pub struct AlignedWeightBlock {
    pub values: [f32; 16],
}

impl AlignedWeightBlock {
    pub fn new() -> Self {
        Self { values: [0.0; 16] }
    }
}

pub fn align_to_64(offset: usize) -> usize {
    (offset + 63) & !63
}

pub struct MmapNodeTable {
    pub ptr: NonNull<u8>,
    pub capacity: usize,
    pub count: usize,
    pub ids_ptr: *mut u64,
    pub type_ids_ptr: *mut u16,
    pub states_ptr: *mut u8,
    pub weights_ptr: *mut f32,
    pub timestamps_ptr: *mut u64,
    pub ext_offsets_ptr: *mut u32,
}

impl MmapNodeTable {
    pub fn new_from_mmap(mmap: &mut MmapMut, capacity: usize) -> Self {
        let base_ptr = mmap.as_mut_ptr();
        let ids_offset = 0;
        let type_ids_offset = align_to_64(ids_offset + (capacity * 8));
        let states_offset = align_to_64(type_ids_offset + (capacity * 2));
        let weights_offset = align_to_64(states_offset + (capacity * 1));
        let timestamps_offset = align_to_64(weights_offset + (capacity * 4));
        let ext_offsets_offset = align_to_64(timestamps_offset + (capacity * 8));

        unsafe {
            Self {
                ptr: NonNull::new_unchecked(base_ptr),
                capacity,
                count: 0,
                ids_ptr: base_ptr.add(ids_offset) as *mut u64,
                type_ids_ptr: base_ptr.add(type_ids_offset) as *mut u16,
                states_ptr: base_ptr.add(states_offset) as *mut u8,
                weights_ptr: base_ptr.add(weights_offset) as *mut f32,
                timestamps_ptr: base_ptr.add(timestamps_offset) as *mut u64,
                ext_offsets_ptr: base_ptr.add(ext_offsets_offset) as *mut u32,
            }
        }
    }

    pub fn add_node(&mut self, id: u64, type_id: u16, weight: f32) -> usize {
        debug_assert!(self.count < self.capacity, "NodeTable capacity exceeded");
        let idx = self.count;
        unsafe {
            self.ids_ptr.add(idx).write(id);
            self.type_ids_ptr.add(idx).write(type_id);
            self.states_ptr.add(idx).write(1); // Active
            self.weights_ptr.add(idx).write(weight);
            self.timestamps_ptr.add(idx).write(0); // Placeholder
            self.ext_offsets_ptr.add(idx).write(0);
        }
        self.count += 1;
        idx
    }

    pub fn calculate_mmap_size(capacity: usize) -> usize {
        let ids_offset = 0;
        let type_ids_offset = align_to_64(ids_offset + (capacity * 8));
        let states_offset = align_to_64(type_ids_offset + (capacity * 2));
        let weights_offset = align_to_64(states_offset + (capacity * 1));
        let timestamps_offset = align_to_64(weights_offset + (capacity * 4));
        let ext_offsets_offset = align_to_64(timestamps_offset + (capacity * 8));
        // Calculate the end of the last array (ext_offsets) and align it
        align_to_64(ext_offsets_offset + (capacity * 4))
    }
}

#[derive(Debug)]
pub struct EdgeTable {
    pub source_indices: Vec<u32>,
    pub target_indices: Vec<u32>,
    pub edge_types: Vec<u16>,
    pub edge_weights: Vec<f32>,
    pub capacity: usize,
    pub count: usize,
}

impl EdgeTable {
    pub fn new(capacity: usize) -> Self {
        Self {
            source_indices: Vec::with_capacity(capacity),
            target_indices: Vec::with_capacity(capacity),
            edge_types: Vec::with_capacity(capacity),
            edge_weights: Vec::with_capacity(capacity),
            capacity,
            count: 0,
        }
    }

    pub fn add_edge(&mut self, src: u32, tgt: u32, edge_type: u16, weight: f32) -> usize {
        debug_assert!(self.count < self.capacity, "EdgeTable capacity exceeded");
        let idx = self.count;
        self.source_indices.push(src);
        self.target_indices.push(tgt);
        self.edge_types.push(edge_type);
        self.edge_weights.push(weight);
        self.count += 1;
        idx
    }
}

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
        let mut table = MmapNodeTable::new_from_mmap(&mut mmap, capacity);
        
        let idx = table.add_node(12345, 1, 0.85);
        assert_eq!(idx, 0);
        assert_eq!(table.count, 1);
        
        unsafe {
            assert_eq!(*table.ids_ptr.add(idx), 12345);
            assert_eq!(*table.weights_ptr.add(idx), 0.85);
            assert_eq!(*table.states_ptr.add(idx), 1);
        }
        
        let _ = std::fs::remove_file("test_node_table.bin");
    }

    #[test]
    fn test_edge_table_addition() {
        let mut table = EdgeTable::new(1024);
        let idx = table.add_edge(0, 1, 5, 0.5);
        assert_eq!(table.source_indices[idx], 0);
        assert_eq!(table.target_indices[idx], 1);
    }

    #[test]
    fn test_mmap_initialization() {
        let _ = std::fs::remove_file("test_graph.bin");
        let result = init_shared_memory("test_graph.bin", 1024 * 1024);
        assert!(result.is_ok());
        // cleanup
        let _ = std::fs::remove_file("test_graph.bin");
    }
}
