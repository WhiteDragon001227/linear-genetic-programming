use std::fmt::Display;

use crate::{
    metrics::definitions::Metric,
    utils::{
        common_traits::{Compare, Show},
        random::generator,
    },
};
use rand::distributions::uniform::{UniformInt, UniformSampler};
use serde::Serialize;

use crate::{
    metrics::accuracy::Accuracy,
    utils::common_traits::{Executables, Inputs},
};

use super::{
    characteristics::{Fitness, FitnessScore, Generate, Organism},
    chromosomes::{Instruction, InstructionGenerateParams},
    registers::{Registers, ValidInput},
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Program<'a, InputType>
where
    InputType: ValidInput,
{
    pub instructions: Vec<Instruction<'a>>,
    pub inputs: &'a Inputs<InputType>,
    pub registers: Registers,
    fitness: Option<FitnessScore>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ProgramGenerateParams<'a, InputType>
where
    InputType: ValidInput,
{
    pub inputs: &'a Inputs<InputType>,
    pub max_instructions: usize,
    #[serde(skip_serializing)]
    pub executables: Executables,
}

impl<'a, InputType> Show for ProgramGenerateParams<'a, InputType> where InputType: ValidInput {}

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

    fn generate(parameters: &Self::GenerateParamsType) -> Self {
        let &ProgramGenerateParams {
            max_instructions,
            inputs,
            executables,
        } = parameters;

        let registers = Registers::new(InputType::N_CLASSES + 1);

        let n_instructions = UniformInt::<usize>::new(0, max_instructions).sample(&mut generator());

        let instruction_params = InstructionGenerateParams::new(
            InputType::N_CLASSES,
            InputType::N_FEATURES,
            executables,
        );

        let instructions: Vec<Instruction> = (0..n_instructions)
            .map(|_| Instruction::generate(&instruction_params))
            .collect();

        Program {
            instructions,
            registers,
            inputs,
            fitness: None,
        }
    }
}

impl<'a, InputType> Fitness for Program<'a, InputType>
where
    InputType: ValidInput,
{
    fn eval_fitness(&self) -> FitnessScore {
        let inputs = self.inputs;

        let mut fitness: Accuracy<Option<usize>> = Accuracy::new();

        for input in inputs {
            let mut registers = self.registers.clone();

            for instruction in &self.instructions {
                let data = instruction.get_data(&registers, input);
                let input_slice = data.get_slice(instruction.target_index, None);
                let register_slice = registers.get_mut_slice(instruction.source_index, None);
                (instruction.exec.get_fn())(register_slice, input_slice);
            }

            let correct_index = input.get_class();
            let registers_argmax = registers.argmax(InputType::N_CLASSES, correct_index);

            Accuracy::observe(&mut fitness, [registers_argmax, Some(correct_index)]);

            registers.reset();
        }

        fitness.calculate()
    }

    fn eval_set_fitness(&mut self) -> FitnessScore {
        *self.fitness.get_or_insert(self.eval_fitness())
    }

    fn get_fitness(&self) -> Option<FitnessScore> {
        self.fitness
    }
}

impl<'a, InputType> Show for Program<'a, InputType> where InputType: ValidInput {}

impl<'a, InputType> Compare for Program<'a, InputType> where InputType: ValidInput {}

impl<'a, InputType> Organism for Program<'a, InputType>
where
    InputType: ValidInput,
{
    fn get_instructions(&self) -> &[crate::genes::chromosomes::Instruction] {
        &self.instructions
    }
}
