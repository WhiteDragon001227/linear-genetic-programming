use crate::{registers::RegisterValue, utils::Show};

pub trait Fitness: Show {
    fn eval_fitness(&self) -> FitnessScore;
}

pub trait Breed: Show {
    fn crossover() -> () {}
}

pub trait Mutate: Show {
    fn mutate() -> () {}
}

pub type FitnessScore = RegisterValue;
