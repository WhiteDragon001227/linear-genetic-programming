use crate::inputs::Inputs;
use crate::program::Program;
use crate::registers::RegisterRepresentable;
use std::collections::VecDeque;
use std::path::Path;

type InnerPopulation<'a, InputType> = VecDeque<Program<'a, InputType>>;
#[derive(Debug, Clone)]
pub struct Population<'a, InputType>(InnerPopulation<'a, InputType>, usize)
where
    InputType: RegisterRepresentable;

impl<'a, InputType> Population<'a, InputType>
where
    InputType: RegisterRepresentable,
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

#[derive(Clone, Copy, Debug)]
pub struct HyperParameters {
    pub population_size: usize,
    pub instruction_size: usize,
    pub retention_rate: f32,
}

pub trait GeneticAlgorithm<'a>
where
    Self::InputType: RegisterRepresentable,
{
    type InputType;

    fn load_inputs(file_path: &'a Path) -> Inputs<Self::InputType>;

    fn new(hyper_params: HyperParameters, inputs: &'a Inputs<Self::InputType>) -> Self;

    fn init_population(&mut self) -> &mut Self;

    fn eval_population(&mut self) -> &mut Self;

    fn apply_natural_selection(&mut self) -> &mut Self;

    fn breed(&mut self) -> &mut Self;
}
