use num_derive::FromPrimitive;
use rand::distributions::uniform::{UniformInt, UniformSampler};
use rand::prelude::{SliceRandom, StdRng};
use rand::{distributions::Standard, prelude::Distribution};
use rand::{thread_rng, Rng, SeedableRng};
use serde::Serialize;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;
use strum::EnumCount;

use crate::utils::alias::{AnyExecutable, Executables};
use crate::utils::random::GENERATOR;

use super::characteristics::Generate;
use super::registers::{Registers, ValidInput};

#[derive(FromPrimitive, Clone, Debug, EnumCount, PartialEq, Eq, Serialize)]
pub enum Modes {
    Input = 0,
    Registers = 1,
}

impl Distribution<Modes> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Modes {
        let should_read_from_input: bool = rng.gen();

        if should_read_from_input {
            return Modes::Input;
        } else {
            return Modes::Registers;
        }
    }
}

#[derive(Clone, Serialize)]
pub struct Instruction {
    pub source_index: usize,
    pub target_index: usize,
    mode: Modes,
    #[serde(skip_serializing)]
    pub exec: AnyExecutable,
}

impl Generate for Instruction {
    type GenerateParamsType = InstructionGenerateParams;

    fn generate(parameters: Option<Self::GenerateParamsType>) -> Self {
        assert!(parameters.is_some());

        let InstructionGenerateParams {
            data_len,
            registers_len,
            executables,
        } = parameters.unwrap();

        let source_index = UniformInt::<usize>::new(0, registers_len).sample(GENERATOR);
        let mode = StdRng::from_entropy().sample(Standard);
        let target_index = UniformInt::<usize>::new(
            0,
            if mode == Modes::Input {
                data_len
            } else {
                registers_len
            },
        )
        .sample(&mut thread_rng());
        let exec = executables.choose(GENERATOR).unwrap();

        Instruction {
            source_index,
            target_index,
            exec: *exec,
            mode,
        }
    }
}
pub struct InstructionGenerateParams {
    registers_len: usize,
    data_len: usize,
    executables: Executables,
}

impl InstructionGenerateParams {
    pub fn new(registers_len: usize, data_len: usize, executables: Executables) -> Option<Self> {
        Some(InstructionGenerateParams {
            registers_len,
            data_len,
            executables,
        })
    }
}

impl Eq for Instruction {}

impl PartialEq for Instruction {
    fn eq(&self, other: &Self) -> bool {
        self.source_index == other.source_index
            && self.target_index == other.target_index
            && self.mode == other.mode
            && self.exec.get_fn() as usize == other.exec.get_fn() as usize
    }
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Instruction")
            .field("mode", &self.mode)
            .field("source_index", &self.source_index)
            .field("target_index", &self.target_index)
            .finish()
    }
}

impl Instruction {
    pub fn get_data<'a, InputType>(
        &self,
        registers: &'a Registers,
        data: &'a InputType,
    ) -> Registers
    where
        InputType: ValidInput,
    {
        let target_data: Registers = match self.mode {
            Modes::Registers => registers.clone(),
            Modes::Input => data.as_registers().clone(),
        };

        target_data
    }
}
