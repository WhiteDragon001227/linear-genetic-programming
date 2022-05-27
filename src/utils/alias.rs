use core::fmt;
use std::fmt::Display;
use std::hash::Hash;

use serde::Serialize;

use crate::genes::internal_repr::RegisterValue;

use super::containers::CollectionIndexPair;

pub type AnyExecutable =
    for<'r, 's> fn(&'r CollectionIndexPair, &'s CollectionIndexPair) -> RegisterValue;

pub trait Compare<V = Self> = PartialEq<V> + Eq + PartialOrd + Ord + Hash where V: Clone;
pub trait Show = fmt::Debug + Display + Serialize;

pub type Inputs<InputType> = Vec<InputType>;
