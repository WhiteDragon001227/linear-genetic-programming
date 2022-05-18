use crate::collection::CollectionIndexPair;
use crate::registers::RegisterValue;

pub type AnyExecutable = fn(CollectionIndexPair, CollectionIndexPair) -> RegisterValue;
