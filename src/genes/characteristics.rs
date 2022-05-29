use crate::utils::alias::Show;

use super::internal_repr::RegisterValue;

pub trait Fitness: Show {
    fn eval(&self) -> FitnessScore;
}

pub trait Breed: Show {
    fn crossover(&self, other: &Self) -> Self;
}

pub trait Mutate: Show {
    fn mutate(&mut self) -> () {}
}

pub trait Generate<T> {
    fn generate(parameters: T) -> Self;
}

pub trait Meta {
    fn get_number_of_features() -> usize;
    fn get_number_of_classes() -> usize;
}

pub type FitnessScore = RegisterValue;
