use crate::containers::CollectionIndexPair;
use crate::registers::RegisterValue;

pub type AnyExecutable =
  for<'r, 's> fn(&'r CollectionIndexPair, &'s CollectionIndexPair) -> RegisterValue;
