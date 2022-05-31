use std::collections::VecDeque;

use crate::{metrics::benchmarks::Benchmark, utils::common_traits::Compare};

use super::characteristics::{FitnessScore, Organism};

type InnerPopulation<T> = VecDeque<T>;
#[derive(Debug, Clone)]
pub struct Population<T>(pub InnerPopulation<T>, usize)
where
    T: Compare;

impl<T> Population<T>
where
    T: Compare,
{
    pub fn new(population_size: usize) -> Self {
        let collection = VecDeque::with_capacity(population_size);
        Population(collection, population_size)
    }

    pub fn get_mut_pop(&mut self) -> &mut InnerPopulation<T> {
        &mut self.0
    }

    pub fn get_pop(&self) -> &InnerPopulation<T> {
        &self.0
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.0.get(index)
    }

    pub fn sort(&mut self) -> () {
        self.0.make_contiguous().sort();
    }

    pub fn first(&self) -> Option<&T> {
        self.0.get(0)
    }

    pub fn last(&self) -> Option<&T> {
        self.0.get(self.0.len() - 1)
    }

    pub fn middle(&self) -> Option<&T> {
        self.0
            .get(math::round::floor(self.0.len() as f64 / 2f64, 1) as usize)
    }

    pub fn f_push(&mut self, value: T) -> () {
        self.0.push_front(value)
    }

    pub fn f_pop(&mut self) -> () {
        self.0.pop_front();
    }

    pub fn push(&mut self, value: T) -> () {
        self.0.push_back(value)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn capacity(&self) -> usize {
        self.1
    }
}

impl<T> Benchmark for Population<T>
where
    T: Organism,
{
    type InputType = FitnessScore;

    fn get_worst(&self) -> Option<Self::InputType> {
        self.first().unwrap().get_fitness()
    }

    fn get_median(&self) -> Option<Self::InputType> {
        self.middle().unwrap().get_fitness()
    }

    fn get_best(&self) -> Option<Self::InputType> {
        self.last().unwrap().get_fitness()
    }
}
