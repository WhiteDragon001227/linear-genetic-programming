use crate::containers::CollectionIndexPair;
use crate::registers::RegisterValue;

pub type AnyExecutable = fn(CollectionIndexPair, CollectionIndexPair) -> RegisterValue;
