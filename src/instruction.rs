use crate::containers::CollectionIndexPair;
use crate::registers::{RegisterRepresentable, RegisterValue, Registers};
use crate::utils::AnyExecutable;
use num_derive::FromPrimitive;
use rand::distributions::uniform::{UniformInt, UniformSampler};
use rand::prelude::{SliceRandom, StdRng};
use rand::{distributions::Standard, prelude::Distribution};
use rand::{thread_rng, Rng, SeedableRng};
use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;
use strum::EnumCount;

impl Debug for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

#[derive(FromPrimitive, Clone, Debug, EnumCount, PartialEq, Eq)]
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

#[derive(Clone, PartialEq, Eq)]
pub struct Instruction {
    source_index: usize,
    target_index: usize,
    mode: Modes,
    exec: AnyExecutable,
}

impl Instruction {
    pub fn apply(
        &self,
        registers: CollectionIndexPair,
        data: CollectionIndexPair,
    ) -> RegisterValue {
        // let CollectionIndexPair(internal_registers, source_index) = registers;
        let value = (self.exec)(registers, data);
        value
    }

    pub fn get_data<'a, T>(
        &self,
        registers: &'a Registers,
        input: &'a T,
    ) -> [CollectionIndexPair; 2]
    where
        T: RegisterRepresentable + Clone,
    {
        let target_data = match &self.mode {
            Modes::Input => input.clone().into(),
            Modes::Registers => registers.clone(),
            _ => unreachable!("This should never happen."),
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

        let source_index = UniformInt::<usize>::new(0, registers_len).sample(&mut thread_rng());
        let target_index = UniformInt::<usize>::new(0, data_len).sample(&mut thread_rng());
        let exec = executables.choose(&mut thread_rng()).unwrap();
        let mode = StdRng::from_entropy().sample(Standard);

        Instruction {
            source_index,
            target_index,
            exec: *exec,
            mode,
        }
    }
}
