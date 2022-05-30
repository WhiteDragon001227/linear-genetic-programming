use core::fmt;
use std::fmt::Display;

use serde::Serialize;

use crate::genes::registers::RegisterValue;

pub type AnyExecutable =
    for<'r, 's> fn(&'r mut [RegisterValue], &'s [RegisterValue]) -> &'r [RegisterValue];

pub type Executables = &'static [AnyExecutable];

pub trait Compare<V = Self> = PartialEq<V> + Eq + PartialOrd + Ord where V: Clone;
pub trait Show = fmt::Debug + Display + Serialize;

pub type Inputs<InputType> = Vec<InputType>;
