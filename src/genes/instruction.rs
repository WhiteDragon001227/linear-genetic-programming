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

use crate::utils::alias::AnyExecutable;
use crate::utils::containers::CollectionIndexPair;
use crate::utils::random::GENERATOR;

use super::internal_repr::{Registers, ValidInput};

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
    source_index: usize,
    target_index: usize,
    mode: Modes,
    #[serde(skip_serializing)]
    exec: AnyExecutable,
}

impl Eq for Instruction {}

impl PartialEq for Instruction {
    fn eq(&self, other: &Self) -> bool {
        self.source_index == other.source_index
            && self.target_index == other.target_index
            && self.mode == other.mode
            && self.exec as usize == other.exec as usize
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
    pub fn apply(
        &self,
        registers: &mut Registers,
        source_data: CollectionIndexPair,
        target_data: CollectionIndexPair,
    ) -> () {
        let value = (self.exec)(&source_data, &target_data);
        let index = source_data.get_index();
        registers.update(index, value);
    }

    pub fn get_data<InputType>(
        &self,
        registers: &Registers,
        input: &InputType,
    ) -> [CollectionIndexPair; 2]
    where
        InputType: ValidInput + Clone,
    {
        let target_data = match &self.mode {
            Modes::Input => input.clone().into(),
            Modes::Registers => registers.clone(),
        };

        let target_data = CollectionIndexPair::new(target_data, self.target_index);
        let source_data = CollectionIndexPair::new(registers.clone(), self.source_index);

        let data = [source_data, target_data];
        data
    }

    pub fn generate(registers_len: usize, data_len: usize, executables: &[AnyExecutable]) -> Self {
        // Sanity check
        assert!(executables.len() != 0);
        assert!(registers_len != 0);
        assert!(data_len != 0);

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
        // update target index

        Instruction {
            source_index,
            target_index,
            exec: *exec,
            mode,
        }
    }
}
