#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
pub struct AlignedWeightBlock {
    pub values: [f32; 16],
}

impl AlignedWeightBlock {
    pub fn new() -> Self {
        Self { values: [0.0; 16] }
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
}
