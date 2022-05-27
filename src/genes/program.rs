use std::fmt::Display;

use serde::Serialize;

use crate::utils::alias::Inputs;

use super::{
    characteristics::{Breed, Fitness, FitnessScore, Generate, Mutate},
    instruction::Instruction,
    internal_repr::{RegisterRepresentable, Registers},
};

trait Organism<T> = Fitness + Breed + Mutate + Generate<T>;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Program<'a, InputType>
where
    InputType: RegisterRepresentable,
{
    pub instructions: Vec<Instruction>,
    pub inputs: &'a Inputs<InputType>,
    pub registers: Registers,
    pub fitness: Option<FitnessScore>,
}

impl<'a, InputType> Display for Program<'a, InputType>
where
    InputType: RegisterRepresentable,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let serialized = toml::to_string(&self).unwrap();
        f.write_str(&serialized)
    }
}

impl<'a, InputType> Ord for Program<'a, InputType>
where
    InputType: RegisterRepresentable,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.fitness.cmp(&other.fitness)
    }
}

impl<'a, InputType> PartialOrd for Program<'a, InputType>
where
    InputType: RegisterRepresentable,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.fitness.partial_cmp(&other.fitness)
    }
}
