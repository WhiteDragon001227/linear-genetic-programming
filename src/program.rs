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
    pub inputs: &'a Inputs<'a, InputType>,
    pub registers: Registers,
    pub fitness: Option<FitnessScore>,
}
