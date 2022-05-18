use crate::registers::Registers;

pub type Collection<ItemType> = Vec<ItemType>;
#[derive(Debug, Clone)]
pub struct CollectionIndexPair(Registers, pub usize);

impl CollectionIndexPair {
    pub fn new(data: Registers, index: usize) -> Self {
        CollectionIndexPair(data, index)
    }
}
