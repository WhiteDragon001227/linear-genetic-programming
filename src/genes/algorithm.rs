use std::path::PathBuf;

use csv::ReaderBuilder;

use crate::utils::alias::{AnyExecutable, Executables, Inputs};

use super::{internal_repr::ValidInput, population::Population, program::Program};

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

    fn init_population<'a>(
        hyper_params: &HyperParameters,
        inputs: &Inputs<Self::InputType>,
    ) -> Population<'a, Self::InputType> {
        let mut population = Population::new(hyper_params.population_size);

        for _ in 0..hyper_params.population_size {
            let program = Program::generate(
                inputs,
                hyper_params.max_program_size,
                hyper_params.executables,
            );
            population.push(program)
        }

        population
    }

    fn evaluate<'a>(
        population: &'a mut Population<'a, Self::InputType>,
    ) -> &'a mut Population<'a, Self::InputType>;

    fn rank<'a>(
        population: &'a mut Population<'a, Self::InputType>,
    ) -> &'a mut Population<'a, Self::InputType>;

    fn apply_selection<'a>(
        population: &'a mut Population<'a, Self::InputType>,
    ) -> &'a mut Population<'a, Self::InputType>;

    fn breed<'a>(
        population: &'a mut Population<'a, Self::InputType>,
    ) -> &'a mut Population<'a, Self::InputType>;

    fn execute(data: &impl Into<PathBuf>) -> () {}
}
