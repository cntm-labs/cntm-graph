use memmap2::MmapMut;
use std::fs::OpenOptions;
use std::io::Result;
use std::ptr::NonNull;

pub fn init_shared_memory(path: &str, size: usize) -> Result<MmapMut> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
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

#[derive(Debug)]
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
    /// # Safety
    /// The `base_ptr` must point to a valid memory region of at least `calculate_mmap_size(capacity)` bytes.
    pub unsafe fn new_from_ptr(base_ptr: *mut u8, capacity: usize) -> Self {
        let count = unsafe { *(base_ptr as *mut u64) as usize };
        let ids_offset = align_to_64(8);
        let type_ids_offset = align_to_64(ids_offset + (capacity * 8));
        let states_offset = align_to_64(type_ids_offset + (capacity * 2));
        let weights_offset = align_to_64(states_offset + capacity);
        let timestamps_offset = align_to_64(weights_offset + (capacity * 4));
        let ext_offsets_offset = align_to_64(timestamps_offset + (capacity * 8));

        unsafe {
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

    pub fn add_node(&mut self, id: u64, type_id: u16, weight: f32) -> usize {
        debug_assert!(self.count < self.capacity, "MmapNodeTable capacity exceeded");
        let idx = self.count;
        unsafe {
            self.ids_ptr.add(idx).write(id);
            self.type_ids_ptr.add(idx).write(type_id);
            self.states_ptr.add(idx).write(1); // Active
            self.weights_ptr.add(idx).write(weight);
            self.timestamps_ptr.add(idx).write(0); // Placeholder
            self.ext_offsets_ptr.add(idx).write(0);

            self.count += 1;
            *(self.ptr.as_ptr() as *mut u64) = self.count as u64;
        }
        idx
    }

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

#[derive(Debug)]
pub struct MmapEdgeTable {
    pub ptr: NonNull<u8>,
    pub capacity: usize,
    pub count: usize,
    pub src_ptr: *mut u32,
    pub tgt_ptr: *mut u32,
    pub types_ptr: *mut u16,
    pub weights_ptr: *mut f32,
}

impl MmapEdgeTable {
    /// # Safety
    /// The `base_ptr` must point to a valid memory region of at least `calculate_mmap_size(capacity)` bytes.
    pub unsafe fn new_from_ptr(base_ptr: *mut u8, capacity: usize) -> Self {
        let count = unsafe { *(base_ptr as *mut u64) as usize };
        let src_offset = align_to_64(8);
        let tgt_offset = align_to_64(src_offset + (capacity * 4));
        let types_offset = align_to_64(tgt_offset + (capacity * 4));
        let weights_offset = align_to_64(types_offset + (capacity * 2));

        unsafe {
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

    pub fn add_edge(&mut self, src: u32, tgt: u32, edge_type: u16, weight: f32) -> usize {
        debug_assert!(self.count < self.capacity, "MmapEdgeTable capacity exceeded");
        let idx = self.count;
        unsafe {
            self.src_ptr.add(idx).write(src);
            self.tgt_ptr.add(idx).write(tgt);
            self.types_ptr.add(idx).write(edge_type);
            self.weights_ptr.add(idx).write(weight);

            self.count += 1;
            *(self.ptr.as_ptr() as *mut u64) = self.count as u64;
        }
        idx
    }

    pub fn calculate_mmap_size(capacity: usize) -> usize {
        let src_offset = align_to_64(8);
        let tgt_offset = align_to_64(src_offset + (capacity * 4));
        let types_offset = align_to_64(tgt_offset + (capacity * 4));
        let weights_offset = align_to_64(types_offset + (capacity * 2));
        align_to_64(weights_offset + (capacity * 4))
    }
}

pub struct GraphStore {
    pub _mmap: MmapMut,
    pub nodes: MmapNodeTable,
    pub edges: MmapEdgeTable,
}

impl GraphStore {
    pub fn new(path: &str, node_cap: usize, edge_cap: usize) -> Result<Self> {
        let nodes_size = MmapNodeTable::calculate_mmap_size(node_cap);
        let edges_size = MmapEdgeTable::calculate_mmap_size(edge_cap);
        let total_size = nodes_size + edges_size;

        let mut mmap = init_shared_memory(path, total_size)?;
        let base_ptr = mmap.as_mut_ptr();

        let nodes = unsafe { MmapNodeTable::new_from_ptr(base_ptr, node_cap) };
        let edges = unsafe { MmapEdgeTable::new_from_ptr(base_ptr.add(nodes_size), edge_cap) };

        Ok(Self {
            _mmap: mmap,
            nodes,
            edges,
        })
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
        let mut table = unsafe { MmapNodeTable::new_from_ptr(mmap.as_mut_ptr(), capacity) };

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
    fn test_mmap_edge_table_addition() {
        let _ = std::fs::remove_file("test_edge_table.bin");
        let capacity = 1024;
        let size = MmapEdgeTable::calculate_mmap_size(capacity);
        let mut mmap = init_shared_memory("test_edge_table.bin", size).unwrap();
        let mut table = unsafe { MmapEdgeTable::new_from_ptr(mmap.as_mut_ptr(), capacity) };

        let idx = table.add_edge(10, 20, 3, 0.75);
        assert_eq!(idx, 0);
        assert_eq!(table.count, 1);

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
            unsafe {
                assert_eq!(*store.nodes.ids_ptr, 1);
                assert_eq!(*store.edges.weights_ptr, 0.1);
            }
        }
        let _ = std::fs::remove_file(path);
    }
}
