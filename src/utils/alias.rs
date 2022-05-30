use core::fmt;
use std::fmt::Display;

use serde::Serialize;

use crate::genes::registers::RegisterValue;

#[derive(Clone)]
pub struct AnyExecutable(&'static str, InternalFn);

impl AnyExecutable {
    pub fn new(name: &'static str, internal_fn: InternalFn) -> Self {
        AnyExecutable(name, internal_fn)
    }

    pub fn get_name(&self) -> &'static str {
        &self.0
    }

    pub fn get_fn(&self) -> InternalFn {
        self.1
    }
}

type InternalFn =
    for<'r, 's> fn(&'r mut [RegisterValue], &'s [RegisterValue]) -> &'r [RegisterValue];

impl fmt::Debug for AnyExecutable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("AnyExecutable").field(&self.0).finish()
    }
}

pub type Executables = &'static [AnyExecutable];

pub trait Compare<V = Self> = PartialEq<V> + Eq + PartialOrd + Ord where V: Clone;
pub trait Show = fmt::Debug + Display + Serialize;

pub type Inputs<InputType> = Vec<InputType>;
