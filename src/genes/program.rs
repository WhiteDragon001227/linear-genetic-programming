use std::fmt::Display;

use rand::distributions::uniform::{UniformInt, UniformSampler};
use serde::Serialize;

use crate::utils::{
    alias::{Executables, Inputs},
    random::GENERATOR,
};

use super::{
    characteristics::{FitnessScore, Generate},
    chromosomes::Instruction,
    registers::{Registers, ValidInput},
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Program<'a, InputType>
where
    InputType: ValidInput,
{
    pub instructions: Vec<Instruction>,
    pub inputs: &'a Inputs<InputType>,
    pub registers: Registers,
    pub fitness: Option<FitnessScore>,
}

pub struct ProgramGenerateParams<'a, InputType>
where
    InputType: ValidInput,
{
    inputs: &'a Inputs<InputType>,
    max_instructions: usize,
    executables: Executables,
}

impl<'a, InputType> Display for Program<'a, InputType>
where
    InputType: ValidInput,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let serialized = toml::to_string(&self).unwrap();
        f.write_str(&serialized)
    }
}

impl<'a, InputType> Ord for Program<'a, InputType>
where
    InputType: ValidInput,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.fitness.cmp(&other.fitness)
    }
}

impl<'a, InputType> PartialOrd for Program<'a, InputType>
where
    InputType: ValidInput,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.fitness.partial_cmp(&other.fitness)
    }
}

impl<'a, InputType> Generate for Program<'a, InputType>
where
    InputType: ValidInput,
{
    type GenerateParamsType = ProgramGenerateParams<'a, InputType>;

    fn generate(parameters: Option<Self::GenerateParamsType>) -> Self {
        let ProgramGenerateParams {
            max_instructions,
            inputs,
            executables,
        } = parameters.unwrap();

        let registers = Registers::new(InputType::N_CLASSES + 1);

        let n_instructions = UniformInt::<usize>::new(0, max_instructions).sample(GENERATOR);

        let instructions: Vec<Instruction> = (0..n_instructions)
            .map(|_| Instruction::generate(InputType::N_CLASSES, InputType::N_CLASSES, executables))
            .collect();

        Program {
            instructions,
            registers,
            inputs,
            fitness: None,
        }
    }
}
