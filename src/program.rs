use crate::{
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
    pub instructions: Vec<Instruction>,
    pub inputs: &'a Inputs<InputType>,
    pub registers: Registers,
    pub fitness: Option<FitnessScore>,
}

impl<'a, InputType> Ord for Program<'a, InputType>
where
    InputType: RegisterRepresentable,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.fitness.cmp(&other.fitness)
    }
}

impl<'a, InputType> PartialOrd for Program<'a, InputType>
where
    InputType: RegisterRepresentable,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.fitness.partial_cmp(&other.fitness)
    }
}
