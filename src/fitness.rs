use crate::registers::RegisterValue;
use std::fmt;

pub trait Fitness: fmt::Debug {
    fn eval_fitness(&self) -> FitnessScore;
}

pub type FitnessScore = RegisterValue;
