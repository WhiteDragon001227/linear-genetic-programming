use std::path::PathBuf;

use csv::ReaderBuilder;
use more_asserts::assert_le;
use rand::prelude::IteratorRandom;

use crate::utils::alias::{Executables, Inputs};

use super::{characteristics::Organism, population::Population, registers::ValidInput};

#[derive(Clone)]
pub struct HyperParameters {
    pub population_size: usize,
    pub max_program_size: usize,
    pub gap: f32,
    pub max_generations: usize,
    pub executables: Executables,
}

pub trait GeneticAlgorithm
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

        let inputs: Vec<Self::InputType> = vec![];

        for input in csv_reader.deserialize() {
            let record: Self::InputType = input.unwrap();
            inputs.push(record);
        }

        inputs
    }

    fn init_population<'a, T>(
        hyper_params: &HyperParameters,
        inputs: &Inputs<Self::InputType>,
        program_params: T::GenerateParamsType,
    ) -> Population<T>
    where
        T: Organism,
    {
        let mut population: Population<T> = Population::new(hyper_params.population_size);

        for _ in 0..hyper_params.population_size {
            let program = T::generate(Some(program_params));
            population.push(program)
        }

        population
    }

    fn evaluate<'a, T: Organism>(population: &'a mut Population<T>) -> &'a mut Population<T> {
        for individual in population.get_mut_pop() {
            individual.lazy_retrieve_fitness();
        }

        population
    }

    fn rank<'a, T: Organism>(population: &'a mut Population<T>) -> &'a mut Population<T> {
        population.sort();
        population
    }

    fn apply_selection<'a, T: Organism>(
        population: &'a mut Population<T>,
        gap: f32,
    ) -> &'a mut Population<T> {
        assert!(gap >= 0f32 && gap <= 1f32);

        assert_le!(
            population.first().unwrap().lazy_retrieve_fitness(),
            population.last().unwrap().lazy_retrieve_fitness()
        );

        let pop_len = population.len();

        let lowest_index = ((1f32 - gap) * (pop_len as f32)).floor() as i32 as usize;

        for _ in 0..lowest_index {
            population.f_pop();
        }

        population
    }

    fn breed<'a, T: Organism>(population: &'a mut Population<T>) -> &'a mut Population<T> {
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

    fn execute(data: &impl Into<PathBuf>) -> () {}
}