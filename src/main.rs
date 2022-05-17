use core::fmt;
use iris::iris_data::{IrisClass, IrisInput};
use num_derive::FromPrimitive;
use rand::{
    distributions::{
        uniform::{UniformInt, UniformSampler},
        Standard,
    },
    prelude::{Distribution, StdRng},
    seq::SliceRandom,
    thread_rng, Rng, SeedableRng,
};
use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Formatter},
    marker::PhantomData,
    path::{Path, PathBuf},
};
use strum::EnumCount;

use csv::ReaderBuilder;
use ordered_float::OrderedFloat;

/// Lets describe the steps we're trying to execute.
///
/// First we initialize a population of programs.
///
/// Programs consist of instructions.
///
/// Instructions consist of four things, a source index, a target index, an operation and a mode.
/// Each instruction is executed. The resulting registers are the the "outputs" of the program.
///
///
/// Data can be retrieved from two places; the registers or the inputs (external data, likely from
/// the fs).
///
/// Ex (Input -> (N, M)):
///
/// 0.1, 0.2, 0.3, 0.4, 0.5
/// 0.2, 0.3, 0.4, 0.5, 0.6
/// ...
///
/// Ex (Registers -> (N,)):
///
/// 0.1, 0.2, 0.3, 0.4, 0.5
///
/// As demonstrated above, inputs can expand in another dimension (in the above case, its the #. of
/// rows) as long as they share a dimension (in the above case, its the #. of columns)
///
/// Smoke Test Algorithm:
///     1. Load input data
///     2. Generate programs (instructions, registers, etc..) -- Init Population
///     3. Eval Fitness
///     --
///     4. Drop x% (tournament selection)
///     5. Clone 1 - x % (Pick from the population uniformly)
///     --
///     6. Repeat from 3 until best == median == worst
///
///
/// Notes:
///     Inputs should be referenced. (RC?)
///
/// Fitness Algorithm:
///     For every input:
///         run all instructions
///         -
///         argmax(registers) == correct_val
///         reset registers
///     Fitness Score = # of correct outputs / total.
///
///
/// Linear Genetic Programming -> 1 Runnable -> N Programmable -> Executable  
///
/// Questions Remaining:
///
/// - [] How do verify the integerity of our indices?
/// - [] Uniform Distribution?
///
/// Registers = # of Total Classes + 1
struct TestLGP<'a>(PhantomData<&'a ()>);

type Collection<ItemType> = Vec<ItemType>;

type RegisterValue = OrderedFloat<f32>;

#[derive(Debug, Clone)]
pub struct Registers(Collection<RegisterValue>);

impl Registers {
    pub fn new(n_registers: usize) -> Registers {
        Registers(vec![OrderedFloat(0f32); n_registers])
    }

    pub fn reset(&mut self) -> () {
        let Registers(internal_registers) = self;

        for index in 0..internal_registers.len() {
            internal_registers[index] = OrderedFloat(0f32);
        }
    }

    pub fn len(&self) -> usize {
        let Registers(internal_registers) = &self;
        internal_registers.len()
    }

    pub fn update(&mut self, index: usize, value: RegisterValue) -> () {
        let Registers(internal_values) = self;
        internal_values[index] = value
    }

    /// Returns:
    ///  `desired_index` if argmax is desired_index else None.
    pub fn argmax(&self, n_classes: usize, desired_index: usize) -> Option<usize> {
        let mut arg_lookup: HashMap<OrderedFloat<f32>, HashSet<usize>> = HashMap::new();

        let Registers(registers) = &self;

        for index in 0..n_classes {
            let value = registers.get(index).unwrap();
            if arg_lookup.contains_key(value) {
                arg_lookup.get_mut(value).unwrap().insert(index);
            } else {
                arg_lookup.insert(*registers.get(index).unwrap(), HashSet::from([index]));
            }
        }

        let max_value = arg_lookup.keys().max().unwrap();
        let indices = arg_lookup.get(max_value).unwrap();

        if indices.contains(&desired_index) {
            if indices.len() == 1 {
                return Some(desired_index);
            }
        }

        None
    }
}

trait RegisterRepresentable: fmt::Debug + Into<Registers> + Clone {
    fn get_number_classes() -> usize;
    fn get_number_features() -> usize;
}

type Inputs<'a, InputType> = Collection<InputType>;

trait Auditable: fmt::Debug {
    fn eval_fitness(&mut self) -> FitnessScore;
}

pub struct CollectionIndexPair<'a>(&'a Registers, usize);

type AnyExecutable = fn(CollectionIndexPair, CollectionIndexPair) -> RegisterValue;

trait Runnable<'a>
where
    Self::InputType: RegisterRepresentable,
{
    type InputType;

    fn load_inputs(file_path: &'a Path) -> Inputs<Self::InputType>;

    fn init_population(
        size: usize,
        max_instructions: usize,
        inputs: &'a Inputs<Self::InputType>,
    ) -> Population<'a, Self::InputType>;

    fn compete(population: Population<'a, Self::InputType>) -> Population<'a, Self::InputType>;
}

/// TODO: Program Generation
#[derive(Clone, Debug)]
struct Program<'a, InputType>
where
    InputType: RegisterRepresentable,
{
    instructions: Collection<Instruction>,
    inputs: &'a Inputs<'a, InputType>,
    registers: Registers,
}

mod iris {
    pub mod iris_ops {

        use crate::{AnyExecutable, CollectionIndexPair, RegisterValue};

        pub fn add(registers: CollectionIndexPair, data: CollectionIndexPair) -> RegisterValue {
            ordered_float::OrderedFloat(0.)
        }

        pub fn subtract(
            registers: CollectionIndexPair,
            data: CollectionIndexPair,
        ) -> RegisterValue {
            ordered_float::OrderedFloat(0.)
        }

        pub fn divide(registers: CollectionIndexPair, data: CollectionIndexPair) -> RegisterValue {
            ordered_float::OrderedFloat(0.)
        }

        pub const EXECUTABLES: &'static [AnyExecutable; 3] =
            &[self::add, self::subtract, self::divide];
    }

    pub mod iris_data {
        use core::fmt;

        use ordered_float::OrderedFloat;
        use serde::{
            de::{self, Visitor},
            Deserialize, Deserializer,
        };
        use strum::EnumCount;

        use crate::{RegisterRepresentable, Registers};

        pub const IRIS_DATASET_LINK: &'static str =
            "https://archive.ics.uci.edu/ml/machine-learning-databases/iris/bezdekIris.data";

        #[derive(Debug, Clone, Copy, Eq, PartialEq, EnumCount)]
        pub enum IrisClass {
            Setosa = 0,
            Versicolour = 1,
            Virginica = 2,
        }

        #[derive(Deserialize, Debug, Clone, PartialEq)]
        pub struct IrisInput {
            sepal_length: f32,
            sepal_width: f32,
            petal_length: f32,
            petal_width: f32,
            #[serde(deserialize_with = "IrisInput::deserialize_iris_class")]
            pub class: IrisClass,
        }

        impl RegisterRepresentable for IrisInput {
            fn get_number_classes() -> usize {
                IrisClass::COUNT
            }

            fn get_number_features() -> usize {
                4
            }
        }

        impl Into<Registers> for IrisInput {
            fn into(self) -> Registers {
                return Registers(vec![
                    OrderedFloat(self.sepal_length),
                    OrderedFloat(self.sepal_width),
                    OrderedFloat(self.petal_length),
                    OrderedFloat(self.petal_width),
                ]);
            }
        }

        impl IrisInput {
            fn deserialize_iris_class<'de, D>(deserializer: D) -> Result<IrisClass, D::Error>
            where
                D: Deserializer<'de>,
            {
                const FIELDS: &'static [&'static str] =
                    &["Iris-setosa", "Iris-versicolor", "Iris-virginica"];

                struct IrisClassVisitor;

                impl<'de> Visitor<'de> for IrisClassVisitor {
                    type Value = IrisClass;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str(&FIELDS.join(" or "))
                    }

                    fn visit_str<E>(self, value: &str) -> Result<IrisClass, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "Iris-setosa" => Ok(IrisClass::Setosa),
                            "Iris-versicolor" => Ok(IrisClass::Versicolour),
                            "Iris-virginica" => Ok(IrisClass::Virginica),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_str(IrisClassVisitor)
            }
        }
    }
}

impl<'a> Program<'a, IrisInput> {
    fn generate(inputs: &'a Inputs<IrisInput>, max_instructions: usize) -> Self {
        let register_len = <IrisInput as RegisterRepresentable>::get_number_classes();
        let registers = Registers::new(register_len);
        let input_len = <IrisInput as RegisterRepresentable>::get_number_features();

        let executables = iris::iris_ops::EXECUTABLES;

        let n_instructions =
            UniformInt::<usize>::new(0, max_instructions).sample(&mut thread_rng());

        let instructions: Vec<Instruction> = (0..n_instructions)
            .map(|_| Instruction::generate(register_len, input_len, executables))
            .collect();

        Program {
            instructions,
            registers,
            inputs,
        }
    }
}

#[derive(FromPrimitive, Clone, Debug, EnumCount)]
enum Modes {
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

/// TODO: Instruction Generation
#[derive(Clone)]
struct Instruction {
    source_index: usize,
    target_index: usize,
    mode: Modes,
    exec: AnyExecutable,
}

impl Instruction {
    fn generate(registers_len: usize, data_len: usize, executables: &[AnyExecutable]) -> Self {
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

impl Debug for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

type FitnessScore = RegisterValue;

impl<'a> Auditable for Program<'a, IrisInput> {
    fn eval_fitness(&mut self) -> FitnessScore {
        let inputs = self.inputs;

        let mut fitness = Accuracy(0, 0);

        for input in inputs {
            let registers = &mut self.registers;

            for instruction in &self.instructions {
                let data = match instruction.mode {
                    Modes::Input => input.clone().into(),
                    _ => registers.clone(),
                };

                let value = (instruction.exec)(
                    CollectionIndexPair(registers, instruction.source_index),
                    CollectionIndexPair(&data, instruction.target_index),
                );

                registers.update(instruction.source_index, value);
            }

            let correct_index = input.class as usize;
            let registers_argmax = registers.argmax(IrisClass::COUNT, correct_index);

            fitness.observe(Some(correct_index) == registers_argmax);
            registers.reset();
        }

        fitness.calculate()
    }
}

#[derive(Debug, Clone)]
struct HyperParameters {
    population_size: usize,
    n_generations: i8,
    selection_dropout: f32,
    input_file_path: PathBuf,
}

type Population<'a, InputType> = Collection<Program<'a, InputType>>;

trait Metric {
    type ObservableType;
    type ResultType;

    fn observe(&mut self, value: Self::ObservableType);
    fn calculate(&self) -> Self::ResultType;
}

// n_correct, total
struct Accuracy(i32, i32);

impl Metric for Accuracy {
    type ObservableType = bool;
    type ResultType = FitnessScore;

    fn observe(&mut self, value: Self::ObservableType) {
        let count = match value {
            true => 1,
            _ => 0,
        };

        self.0 += count;
        self.1 += 1
    }

    fn calculate(&self) -> Self::ResultType {
        let Accuracy(n_correct, total) = self;
        OrderedFloat(*n_correct as f32) / OrderedFloat(*total as f32)
    }
}

impl<'a> Runnable<'a> for TestLGP<'a> {
    type InputType = IrisInput;

    fn load_inputs(file_path: &'a Path) -> Inputs<Self::InputType> {
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(false)
            .from_path(file_path)
            .unwrap();

        let raw_inputs: Vec<IrisInput> = csv_reader
            .deserialize()
            .map(|input| -> IrisInput { input.unwrap() })
            .collect();

        return raw_inputs;
    }

    fn compete(population: Population<'a, Self::InputType>) -> Population<'a, Self::InputType> {
        todo!()
    }

    fn init_population(
        size: usize,
        max_instructions: usize,
        inputs: &'a Inputs<Self::InputType>,
    ) -> Population<'a, Self::InputType> {
        (0..size)
            .map(|_| Program::generate(inputs, max_instructions))
            .collect()
    }
}

fn main() {
    // TODO:
    // 1. Load Data
    // 2. Generate Population
    // 3. Run Programs in Population
    // 4. Evaluate Programs
    // 5. Repeat From 3 until N Generations Have Been Created
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use std::{error, io::Write};

    use strum::Display;
    use tempfile::NamedTempFile;

    use crate::iris::iris_data::IRIS_DATASET_LINK;

    use super::*;

    async fn get_iris_content() -> Result<ContentFilePair, Box<dyn error::Error>> {
        let tmp_file = NamedTempFile::new()?;
        let response = reqwest::get(IRIS_DATASET_LINK).await?;
        let content = response.text().await?;
        writeln!(&tmp_file, "{}", &content)?;

        Ok(ContentFilePair(content, tmp_file))
    }

    struct ContentFilePair(String, NamedTempFile);

    #[tokio::test]
    async fn given_inputs_and_hyperparams_when_population_is_initialized_then_population_generated_with_hyperparams_and_inputs(
    ) -> Result<(), Box<dyn error::Error>> {
        let ContentFilePair(_, tmp_file) = get_iris_content().await?;
        let inputs = <TestLGP as Runnable>::load_inputs(tmp_file.path());
        const SIZE: usize = 100;
        const MAX_INSTRUCTIONS: usize = 100;
        let population = <TestLGP as Runnable>::init_population(SIZE, MAX_INSTRUCTIONS, &inputs);

        assert!(population.len() == SIZE);

        for individual in population {
            assert!(individual.instructions.len() <= SIZE)
        }

        Ok(())
    }

    #[tokio::test]
    async fn given_iris_dataset_when_csv_is_read_then_rows_are_deserialized_as_structs(
    ) -> Result<(), Box<dyn error::Error>> {
        let ContentFilePair(content, _) = get_iris_content().await?;
        assert_ne!(content.len(), 0);

        let content_bytes = content.as_bytes();

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(content_bytes);

        let data = reader.deserialize();
        let mut count = 0;

        for result in data {
            let _record: IrisInput = result?;
            count += 1;
        }

        assert_ne!(count, 0);

        Ok(())
    }

    #[tokio::test]
    async fn given_iris_dataset_when_csv_path_is_provided_then_collection_of_iris_structs_are_returned(
    ) -> Result<(), Box<dyn error::Error>> {
        let ContentFilePair(_, tmpfile) = get_iris_content().await?;
        let inputs = <TestLGP as Runnable>::load_inputs(tmpfile.path());
        assert_ne!(inputs.len(), 0);
        Ok(())
    }
}
