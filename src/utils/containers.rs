use crate::genes::registers::{RegisterValue, Registers};

#[derive(Debug, Clone)]
pub struct CollectionIndexPair(Registers, usize);

impl CollectionIndexPair {
    pub fn new(data: Registers, index: usize) -> Self {
        CollectionIndexPair(data, index)
    }

    pub fn get_value(&self) -> RegisterValue {
        let CollectionIndexPair(internal_registers, index) = self;

        internal_registers.get_value_at_index(*index)
    }

    pub fn get_index(&self) -> usize {
        self.1
    }
}
