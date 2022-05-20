use crate::{
    containers::Collection,
    fitness::FitnessScore,
    inputs::Inputs,
    instruction::Instruction,
    registers::{RegisterRepresentable, Registers},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program<'a, InputType>
where
    InputType: RegisterRepresentable,
{
    pub instructions: Collection<Instruction>,
    pub inputs: &'a Inputs<InputType>,
    pub registers: Registers,
    pub fitness: Option<FitnessScore>,
}

impl<'a, InputType> PartialOrd for Program<'a, InputType>
where
    InputType: RegisterRepresentable,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.fitness.partial_cmp(&other.fitness)
    }
}
