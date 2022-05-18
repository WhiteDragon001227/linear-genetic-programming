use crate::{
    collection::Collection,
    inputs::Inputs,
    instruction::Instruction,
    registers::{RegisterRepresentable, Registers},
};

#[derive(Clone, Debug)]
pub struct Program<'a, InputType>
where
    InputType: RegisterRepresentable,
{
    pub instructions: Collection<Instruction>,
    pub inputs: &'a Inputs<'a, InputType>,
    pub registers: Registers,
}
