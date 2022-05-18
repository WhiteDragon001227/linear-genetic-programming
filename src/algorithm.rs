use crate::collection::Collection;
use crate::inputs::Inputs;
use crate::program::Program;
use crate::registers::RegisterRepresentable;
use std::path::Path;

pub type Population<'a, InputType> = Collection<Program<'a, InputType>>;
pub trait GeneticAlgorithm<'a>
where
    Self::InputType: RegisterRepresentable,
{
    type InputType;

    fn load_inputs(file_path: &'a Path) -> Inputs<Self::InputType>;

    fn init_population(
        size: usize,
        max_instructions: usize,
        inputs: &'a Inputs<Self::InputType>,
    ) -> Population<'a, Self::InputType>;

    fn retrieve_selection(
        population: Population<'a, Self::InputType>,
        retention_rate: f32,
    ) -> Population<'a, Self::InputType>;

    fn breed(population: Population<'a, Self::InputType>) -> Population<'a, Self::InputType>;
}
