use core::fmt;
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
    u8,
};

use csv::ReaderBuilder;
use ordered_float::OrderedFloat;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};

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

const IRIS_DATASET_LINK: &'static str =
    "https://archive.ics.uci.edu/ml/machine-learning-databases/iris/bezdekIris.data";

type Collection<ItemType> = Vec<ItemType>;

type RegisterValue = OrderedFloat<f32>;

#[derive(Debug, Clone)]
struct Registers(Collection<RegisterValue>);

impl Registers {
    fn new(n_registers: usize) -> Registers {
        Registers(vec![OrderedFloat(0f32); n_registers])
    }

    fn reset(&mut self) -> () {
        let registers = &mut self.0;

        for index in 1..registers.len() {
            registers[index - 1] = OrderedFloat(0f32);
        }
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn update(&mut self, index: usize, value: RegisterValue) -> () {
        let Registers(internal_values) = self;
        internal_values[index] = value
    }

    /// Returns:
    ///  `desired_index` if argmax is desired_index else None.
    fn argmax(&self, n_classes: usize, desired_index: usize) -> Option<usize> {
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

trait RegisterRepresentable: fmt::Debug + Into<Registers> {}

type Inputs<'a, InputType> = Collection<InputType>;

trait Auditable: fmt::Debug {
    fn eval_fitness(&mut self) -> FitnessScore;
}

// For convenience.
// AnyExecutive => Side Effects...
type AnyExecutable = fn(&Registers, &Registers, usize, i8) -> RegisterValue;
type AnyProgrammable<'a, T> = Box<dyn Programmable<'a, InputType = T> + 'a>;

trait Programmable<'a>: fmt::Debug + Auditable
where
    Self::InputType: RegisterRepresentable,
{
    type InputType;

    fn dyn_clone(&self) -> AnyProgrammable<'a, Self::InputType>;
}

impl<'a, InputType> Clone for AnyProgrammable<'a, InputType>
where
    InputType: RegisterRepresentable,
{
    fn clone(&self) -> Self {
        self.dyn_clone()
    }
}

trait Runnable<'a>
where
    Self::ProgramType: Programmable<'a>,
{
    type ProgramType;

    fn load_inputs(
        file_path: &'a Path,
    ) -> Inputs<<<Self as Runnable<'a>>::ProgramType as Programmable<'a>>::InputType>;

    fn generate_individual(
        inputs: &Inputs<<<Self as Runnable<'a>>::ProgramType as Programmable<'a>>::InputType>,
    ) -> Self::ProgramType;

    fn init_population(size: usize) -> Population<'a, Self::ProgramType>;

    fn compete(
        population: Population<'a, Self::ProgramType>,
        retention_percent: f32,
    ) -> Population<Self::ProgramType>;
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

#[derive(FromPrimitive, Clone, Debug)]
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
    target_index: i8,
    mode: Modes,
    exec: AnyExecutable,
}

impl Instruction {
    fn generate(
        registers_len: usize,
        data_len: usize,
        executables: Vec<AnyExecutable>,
    ) -> Instruction {
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
            target_index: target_index as i8,
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
                    registers,
                    &data,
                    instruction.source_index,
                    instruction.target_index,
                );

                registers.update(instruction.source_index, value);
            }

            let correct_index = input.class as usize;
            let registers_argmax = registers.argmax(N_CLASSES_IRIS, correct_index);

            fitness.observe(Some(correct_index) == registers_argmax);
            registers.reset();
        }

        fitness.calculate()
    }
}

impl<'a> Programmable<'a> for Program<'a, IrisInput> {
    type InputType = IrisInput;

    fn dyn_clone(&self) -> AnyProgrammable<'a, Self::InputType> {
        let clone = Program::<'a, Self::InputType> {
            inputs: &self.inputs,
            instructions: self.instructions.clone(),
            registers: self.registers.clone(),
        };
        Box::new(clone)
    }
}

#[derive(Debug, Clone)]
struct HyperParameters {
    population_size: usize,
    n_generations: i8,
    selection_dropout: f32,
    input_file_path: PathBuf,
}

#[derive(Debug, Clone)]
struct Population<'a, ProgramType>(Collection<ProgramType>, PhantomData<&'a ()>)
where
    ProgramType: Programmable<'a>;

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
        OrderedFloat(self.0 as f32) / OrderedFloat(self.1 as f32)
    }
}

impl<'a> Runnable<'a> for TestLGP<'a> {
    type ProgramType = Program<'a, IrisInput>;

    fn load_inputs(
        file_path: &'a Path,
    ) -> Inputs<<<Self as Runnable<'a>>::ProgramType as Programmable<'a>>::InputType> {
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

    fn generate_individual(inputs: &Inputs<IrisInput>) -> Self::ProgramType {
        /*
         *        let internals = InternalProgram {
         *            registers: Registers::new(N_REGISTERS)
         *        };
         *
         *        return Program {
         *            inputs,
         *            internals,
         *
         *        }
         */
        todo!("Need to randomly generate individual.")
    }

    fn init_population(size: usize) -> Population<'a, Self::ProgramType> {
        todo!()
    }

    fn compete(
        population: Population<'a, Self::ProgramType>,
        retention_percent: f32,
    ) -> Population<Self::ProgramType> {
        todo!()
    }
}

const N_CLASSES_IRIS: usize = 3;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum IrisClass {
    Setosa = 0,
    Versicolour = 1,
    Virginica = 2,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
struct IrisInput {
    sepal_length: f32,
    sepal_width: f32,
    petal_length: f32,
    petal_width: f32,
    #[serde(deserialize_with = "IrisInput::deserialize_iris_class")]
    class: IrisClass,
}

impl RegisterRepresentable for IrisInput {}

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

    use tempfile::NamedTempFile;

    use super::*;

    async fn get_iris_content() -> Result<String, Box<dyn error::Error>> {
        let response = reqwest::get(IRIS_DATASET_LINK).await?;
        let content = response.text().await?;

        Ok(content)
    }

    #[tokio::test]
    async fn given_iris_dataset_when_csv_is_read_then_rows_are_deserialized_as_structs(
    ) -> Result<(), Box<dyn error::Error>> {
        let content = get_iris_content().await?;
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
        let tmpfile = NamedTempFile::new()?;
        let content = get_iris_content().await?;
        writeln!(&tmpfile, "{}", &content)?;
        let inputs = <TestLGP as Runnable>::load_inputs(tmpfile.path());
        assert_ne!(inputs.len(), 0);
        Ok(())
    }
}
