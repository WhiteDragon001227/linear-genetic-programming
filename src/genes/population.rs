use std::collections::VecDeque;

use crate::{genes::internal_repr::ValidInput, genes::program::Program};

type InnerPopulation<'a, InputType> = VecDeque<Program<'a, InputType>>;
#[derive(Debug, Clone)]
pub struct Population<'a, InputType>(InnerPopulation<'a, InputType>, usize)
where
    InputType: ValidInput;

impl<'a, InputType> Population<'a, InputType>
where
    InputType: ValidInput,
{
    pub fn new(population_size: usize) -> Self {
        let collection = VecDeque::with_capacity(population_size);
        Population(collection, population_size)
    }

    pub fn get_mut_pop(&mut self) -> &mut InnerPopulation<'a, InputType> {
        &mut self.0
    }

    pub fn get_pop(&self) -> &InnerPopulation<'a, InputType> {
        &self.0
    }

    pub fn get(&self, index: usize) -> Option<&Program<'a, InputType>> {
        self.0.get(index)
    }

    pub fn sort(&mut self) -> () {
        self.0.make_contiguous().sort();
    }

    pub fn first(&self) -> Option<&Program<'a, InputType>> {
        self.0.get(0)
    }

    pub fn last(&self) -> Option<&Program<'a, InputType>> {
        self.0.get(self.0.len() - 1)
    }

    pub fn middle(&self) -> Option<&Program<'a, InputType>> {
        self.0
            .get(math::round::floor(self.0.len() as f64 / 2f64, 1) as usize)
    }

    pub fn f_push(&mut self, value: Program<'a, InputType>) -> () {
        self.0.push_front(value)
    }

    pub fn f_pop(&mut self) -> () {
        self.0.pop_front();
    }

    pub fn push(&mut self, value: Program<'a, InputType>) -> () {
        self.0.push_back(value)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn capacity(&self) -> usize {
        self.1
    }
}
