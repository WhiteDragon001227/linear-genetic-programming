use std::path::PathBuf;

use csv::ReaderBuilder;
use more_asserts::assert_le;
use rand::prelude::IteratorRandom;

use crate::{
    genes::characteristics::{Fitness, Generate},
    utils::common_traits::Inputs,
};

use super::{characteristics::Organism, population::Population, registers::ValidInput};

#[derive(Clone)]
pub struct HyperParameters<OrganismType>
where
    OrganismType: Organism,
{
    pub population_size: usize,
    pub gap: f32,
    pub max_generations: usize,
    pub program_params: OrganismType::GenerateParamsType,
}

pub trait Loader
where
    Self::InputType: ValidInput,
{
    type InputType;

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
}

pub trait GeneticAlgorithm
where
    Self::O: Organism,
{
    type O;

    fn init_env() -> () {
        pretty_env_logger::init();
    }

    fn init_population(hyper_params: &HyperParameters<Self::O>) -> Population<Self::O> {
        let mut population = Population::new(hyper_params.population_size);

        for _ in 0..hyper_params.population_size {
            let program = Self::O::generate(&hyper_params.program_params);
            population.push(program)
        }

        population
    }

    fn evaluate(population: &mut Population<Self::O>) -> () {
        for individual in population.get_mut_pop() {
            individual.eval_set_fitness();
        }
    }

    fn rank(population: &mut Population<Self::O>) -> () {
        population.sort();
    }

    fn apply_selection(population: &mut Population<Self::O>, gap: f32) -> () {
        assert!(gap >= 0f32 && gap <= 1f32);

        assert_le!(
            population.first().unwrap().eval_fitness(),
            population.last().unwrap().eval_fitness()
        );

        let pop_len = population.len();

        let lowest_index = ((1f32 - gap) * (pop_len as f32)).floor() as i32 as usize;

        for _ in 0..lowest_index {
            population.f_pop();
        }
    }

    fn breed(population: &mut Population<Self::O>) -> () {
        let pop_cap = population.capacity();
        let pop_len = population.len();
        let remaining_size = pop_cap - pop_len;

        let selected_individuals = population
            .get_pop()
            .iter()
            .cloned()
            .choose_multiple(&mut rand::thread_rng(), remaining_size);

        for individual in selected_individuals {
            population.push(individual)
        }
    }

    // TODO: Add hooks?
    fn execute(hyper_params: &HyperParameters<Self::O>) -> Population<Self::O> {
        Self::init_env();

        let mut population = Self::init_population(hyper_params);

        for _ in 0..hyper_params.max_generations {
            Self::apply_selection(&mut population, hyper_params.gap);
            Self::evaluate(&mut population);
            Self::rank(&mut population);
            Self::breed(&mut population);
        }

        population
    }
}
