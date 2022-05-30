use crate::utils::common_traits::{Compare, Show};

use super::{chromosomes::Instruction, registers::RegisterValue};

pub type FitnessScore = RegisterValue;

pub trait Fitness: Show {
    fn fitness(&self) -> FitnessScore;
    fn lazy_fitness(&mut self) -> FitnessScore;
}

pub trait Breed: Show {
    fn crossover(&self, other: &Self) -> Self;
}

pub trait Mutate: Show {
    fn mutate(&mut self) -> () {}
}

pub trait Generate {
    type GenerateParamsType;

    fn generate<'a>(parameters: &'a Self::GenerateParamsType) -> Self;
}

pub trait Organism: Fitness + Generate + Compare {
    fn get_instructions(&self) -> &[Instruction];
}
