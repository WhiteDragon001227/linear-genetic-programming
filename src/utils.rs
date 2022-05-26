use core::fmt;
use std::fmt::Display;
use std::hash::Hash;

use serde::Serialize;

use crate::containers::CollectionIndexPair;
use crate::registers::RegisterValue;

pub type AnyExecutable =
    for<'r, 's> fn(&'r CollectionIndexPair, &'s CollectionIndexPair) -> RegisterValue;

pub trait Compare<V = Self> = PartialEq<V> + Eq + PartialOrd + Ord + Hash where V: Clone;
pub trait Show = fmt::Debug + Display + Serialize;
