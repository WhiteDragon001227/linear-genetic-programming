use std::path::PathBuf;

use csv::ReaderBuilder;
use more_asserts::assert_le;
use rand::prelude::IteratorRandom;

use crate::utils::alias::{Executables, Inputs};

use super::{
    characteristics::Organism, population::Population, program::ProgramGenerateParams,
    registers::ValidInput,
};

#[derive(Clone)]
pub struct HyperParameters<'a, InputType, DataPathType = String>
where
    DataPathType: Into<PathBuf>,
    InputType: ValidInput,
{
    pub population_size: usize,
    pub max_program_size: usize,
    pub gap: f32,
    pub max_generations: usize,
    pub data_path: DataPathType,
    pub executables: Executables,
    pub program_params: ProgramGenerateParams<'a, InputType>,
}

pub trait GeneticAlgorithm<'a>
where
    Self::InputType: ValidInput,
{
    type InputType;

    fn init_env() -> () {
        pretty_env_logger::init();
    }

    fn load_inputs(file_path: impl Into<PathBuf>) -> Inputs<Self::InputType> {
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(false)
            .from_path(file_path.into())
            .unwrap();

        let mut inputs: Vec<Self::InputType> = vec![];

        for input in csv_reader.deserialize() {
            let record: Self::InputType = input.unwrap();
            inputs.push(record);
        }

        inputs
    }

    fn init_population<T>(
        hyper_params: &HyperParameters<Self::InputType>,
        inputs: &Inputs<Self::InputType>,
    ) -> Population<T>
    where
        T: Organism,
    {
        let mut population: Population<T> = Population::new(hyper_params.population_size);

        for _ in 0..hyper_params.population_size {
            let program = T::generate(Some(&hyper_params.program_params));
            population.push(program)
        }

        population
    }

    fn evaluate<'b, T: Organism>(population: &'b mut Population<T>) -> &'b mut Population<T> {
        for individual in population.get_mut_pop() {
            individual.lazy_fitness();
        }

        population
    }

    fn rank<'b, T: Organism>(population: &'b mut Population<T>) -> &'b mut Population<T> {
        population.sort();
        population
    }

    fn apply_selection<'b, T: Organism>(
        population: &'b mut Population<T>,
        gap: f32,
    ) -> &'b mut Population<T> {
        assert!(gap >= 0f32 && gap <= 1f32);

        assert_le!(
            population.first().unwrap().fitness(),
            population.last().unwrap().fitness()
        );

        let pop_len = population.len();

        let lowest_index = ((1f32 - gap) * (pop_len as f32)).floor() as i32 as usize;

        for _ in 0..lowest_index {
            population.f_pop();
        }

        population
    }

    fn breed<'b, T: Organism>(population: &'b mut Population<T>) -> &'b mut Population<T> {
        let pop_cap = population.capacity();
        let pop_len = population.len();
        let remaining_size = pop_cap - pop_len;

        let selected_individuals: Vec<T> = population
            .get_pop()
            .iter()
            .cloned()
            .choose_multiple(&mut rand::thread_rng(), remaining_size);

        for individual in selected_individuals {
            population.push(individual)
        }

        population
    }

    fn execute<T>(
        data_path: &impl Into<PathBuf>,
        program_params: T::GenerateParamsType,
        hyper_params: &HyperParameters<Self::InputType>,
    ) -> ()
    where
        T: Organism,
    {
    }
}
