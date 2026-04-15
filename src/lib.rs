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

#[derive(Debug)]
pub struct NodeTable {
    pub ids: Vec<u64>,
    pub type_ids: Vec<u16>,
    pub states: Vec<u8>,
    pub weights: Vec<f32>,
    pub timestamps: Vec<u64>,
    pub ext_offsets: Vec<u32>,
    pub capacity: usize,
    pub count: usize,
}

impl NodeTable {
    pub fn new(capacity: usize) -> Self {
        Self {
            ids: Vec::with_capacity(capacity),
            type_ids: Vec::with_capacity(capacity),
            states: Vec::with_capacity(capacity),
            weights: Vec::with_capacity(capacity),
            timestamps: Vec::with_capacity(capacity),
            ext_offsets: Vec::with_capacity(capacity),
            capacity,
            count: 0,
        }
    }

    pub fn add_node(&mut self, id: u64, type_id: u16, weight: f32) -> usize {
        debug_assert!(self.count < self.capacity, "NodeTable capacity exceeded");
        let idx = self.count;
        self.ids.push(id);
        self.type_ids.push(type_id);
        self.states.push(1); // Active
        self.weights.push(weight);
        self.timestamps.push(0); // Placeholder
        self.ext_offsets.push(0);
        self.count += 1;
        idx
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
        let mut table = NodeTable::new(1024);
        let idx = table.add_node(12345, 1, 0.85);
        assert_eq!(table.ids[idx], 12345);
        assert_eq!(table.weights[idx], 0.85);
    }

    #[test]
    fn test_edge_table_addition() {
        let mut table = EdgeTable::new(1024);
        let idx = table.add_edge(0, 1, 5, 0.5);
        assert_eq!(table.source_indices[idx], 0);
        assert_eq!(table.target_indices[idx], 1);
    }
}
