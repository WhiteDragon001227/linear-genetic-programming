use crate::registers::Registers;

pub type Collection<ItemType> = Vec<ItemType>;

pub struct CollectionIndexPair<'a>(&'a Registers, usize);
