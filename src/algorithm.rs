use crate::containers::Collection;
use crate::inputs::Inputs;
use crate::program::Program;
use crate::registers::RegisterRepresentable;
use std::path::Path;

pub type Population<'a, InputType> = Collection<Program<'a, InputType>>;
pub type PopulationSlice<'a, InputType> = &'a [Program<'a, InputType>];

pub struct HyperParameters<'a> {
    pub input_path: &'a Path,
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

    fn new(hyper_params: HyperParameters<'a>) -> Self;

    fn init_population(&mut self) -> Self;

    fn eval_population(&mut self) -> Self;

    fn apply_natural_selection(&mut self) -> Self;

    fn breed(&mut self) -> Self;
}
