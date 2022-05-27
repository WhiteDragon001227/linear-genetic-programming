use std::path::PathBuf;

use crate::utils::alias::Inputs;

use super::{internal_repr::RegisterRepresentable, population::Population};

#[derive(Clone, Copy, Debug)]
pub struct HyperParameters {
    pub population_size: usize,
    pub max_program_size: usize,
    pub gap: f32,
    pub max_generations: usize,
    pub data_path: String,
}

pub trait GeneticAlgorithm<'a>
where
    Self::InputType: RegisterRepresentable,
{
    type InputType;

    fn init_env() -> () {
        pretty_env_logger::init();
    }

    fn load_inputs(file_path: impl Into<PathBuf>) -> Inputs<Self::InputType>;

    fn init_population(hyper_params: &HyperParameters) -> Population<Self::InputType>;

    fn evaluate(population: &mut Population<Self::InputType>) -> ();

    fn rank(&mut self) -> &mut Self;

    fn apply_selection(&mut self) -> &mut Self;

    fn breed(&mut self) -> &mut Self;

    fn execute(data: &impl Into<PathBuf>) -> () {}
}
