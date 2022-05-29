use crate::utils::alias::{Compare, Show};

use super::{chromosomes::Instruction, registers::RegisterValue};

pub type FitnessScore = RegisterValue;
pub trait Fitness: Show {
    fn retrieve_fitness(&self) -> FitnessScore;
    fn lazy_retrieve_fitness(&mut self) -> ();
}

pub trait Breed: Show {
    fn crossover(&self, other: &Self) -> Self;
}

pub trait Mutate: Show {
    fn mutate(&mut self) -> () {}
}

pub trait Generate {
    type GenerateParamsType;
    fn generate(parameters: Option<Self::GenerateParamsType>) -> Self;
}

pub trait Meta {
    fn get_number_of_features() -> usize;
    fn get_number_of_classes() -> usize;
}

pub trait Organism: Fitness + Breed + Mutate + Generate + Compare {
    fn get_instructions(&self) -> &[Instruction];
}
