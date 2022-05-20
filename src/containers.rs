use crate::registers::{RegisterValue, Registers};

pub type Collection<ItemType> = Vec<ItemType>;

#[derive(Debug, Clone)]
pub struct CollectionIndexPair(pub Registers, pub usize);

impl CollectionIndexPair {
    pub fn new(data: Registers, index: usize) -> Self {
        CollectionIndexPair(data, index)
    }

    pub fn get_value(&self) -> RegisterValue {
        let CollectionIndexPair(internal_registers, index) = self;

        internal_registers.get_value_at_index(*index)
    }
}
