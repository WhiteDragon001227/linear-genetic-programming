use core::fmt;
use num::FromPrimitive;
use num_derive::FromPrimitive;
use std::{
    fmt::{Debug, Formatter},
    marker::PhantomData,
    path::{Path, PathBuf},
    u8,
};

use csv::ReaderBuilder;

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
///     4. Drop x% (tournament selection)
///     5. Clone 1 - x %
///     6. Repeat from 3 until best == median == worst
///
///
/// Notes:
///     Inputs should be referenced. (RC?)
///
/// Fitness Algorithm:
///     For every input:
///         run all instructions
///         argmax(instructions) == correct_val
///         reset registers
///     Fitness Score = # of correct / # of inputs.
///
///
/// Linear Genetic Programming -> 1 Runnable -> N Programmable -> Executable  
struct TestLGP<'a>(PhantomData<&'a ()>);

const IRIS_DATASET_LINK: &'static str =
    "https://archive.ics.uci.edu/ml/machine-learning-databases/iris/bezdekIris.data";

type Collection<ItemType> = Vec<ItemType>;

#[derive(Debug, Clone)]
struct Registers(Collection<f32>);

impl Registers {
    fn new(n_registers: usize) -> Registers {
        Registers(vec![0.; n_registers])
    }

    fn reset(&mut self) -> () {
        let registers = &mut self.0;

        for index in 1..registers.len() {
            registers[index - 1] = 0.;
        }
    }

    fn argmax<T: FromPrimitive>(&self) -> Option<T> {
        let mut max_index: i32 = -1;
        let Registers(registers) = &self;
        let mut current_max = f32::NEG_INFINITY;

        for (index, value) in registers.iter().enumerate() {
            if value > &current_max {
                current_max = *value;
                max_index = index as i32;
            }
        }

        num::FromPrimitive::from_i32(max_index)
    }
}

trait RegisterRepresentable: fmt::Debug + Into<Registers>
where
    Self::TrueType: FromPrimitive,
{
    type TrueType;

    fn argmax(registers: Registers) -> Option<Self::TrueType>;
}

#[derive(Debug, Clone)]
struct Inputs<InputType: RegisterRepresentable>(Collection<InputType>);

trait Auditable: fmt::Debug {
    fn eval_fitness(&mut self) -> f32;
}

struct SourceIndex(i8);
struct TargetIndex(i8);

// For convenience.
type AnyExecutable<'a, T> = fn(&Registers, &Data<'a, T>, SourceIndex, TargetIndex) -> Registers;
type AnyProgrammable<'a, T> = Box<dyn Programmable<'a, InputType = T> + 'a>;

#[derive(Debug, Clone)]
enum Data<'a, InputType>
where
    InputType: RegisterRepresentable,
{
    Input(&'a InputType),
    Registers(&'b Registers),
}

trait Programmable<'a>: fmt::Debug + Auditable
where
    Self::InputType: RegisterRepresentable,
{
    type InputType;

    fn get_inputs(&self) -> &'a Inputs<Self::InputType>;
    fn get_instructions(&self) -> &Collection<AnyExecutable<Self::InputType>>;
    fn get_registers(&mut self) -> &mut Registers;

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

#[derive(Clone, Debug)]
struct Instruction<'a, InputType>
where
    InputType: RegisterRepresentable,
{
    source: i8,
    target: i8,
    data: Data<'a, InputType>,
}

#[derive(Clone)]
struct Program<'a, InputType>
where
    InputType: RegisterRepresentable,
{
    instructions: Collection<AnyExecutable<InputType>>,
    inputs: &'a Inputs<InputType>,
    internals: InternalProgram,
}

impl<'a, T> fmt::Debug for Program<'a, T>
where
    T: RegisterRepresentable,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let instructions = &self.instructions.iter().map(|x| -> &fn(&'a Registers, ) )
        f.debug_struct("Program")
            .field("inputs", &self.inputs)
            .field("internals", &self.internals)
        .finish();
    }
}

#[derive(Debug, Clone)]
struct InternalProgram {
    registers: Registers,
}

impl<'a> Auditable for Program<'a, IrisInput> {
    fn eval_fitness(&mut self) -> f32 {
        let Inputs(inputs) = self.get_inputs();

        for input in inputs {
            let mut registers = self.get_registers();
            for instruction in self.get_instructions() {
                //instruction.exec(&mut registers)
            }

            /*
             *registers.reset();
             */

            // reset
            // count - metrics
        }

        0.
    }
}

impl<'a> Programmable<'a> for Program<'a, IrisInput> {
    type InputType = IrisInput;

    fn get_inputs(&self) -> &'a Inputs<Self::InputType> {
        &self.inputs
    }

    fn get_instructions(&self) -> &Collection<AnyExecutable<Self::InputType>> {
        return &self.instructions;
    }

    fn dyn_clone(&self) -> AnyProgrammable<'a, Self::InputType> {
        let clone = Program::<'a, IrisInput> {
            inputs: &self.inputs,
            instructions: self.instructions.clone(),
            internals: self.internals.clone(),
        };
        Box::new(clone)
    }

    fn get_registers(&mut self) -> &mut Registers {
        &mut self.internals.registers
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

        return Inputs(raw_inputs);
    }

    fn generate_individual(inputs: &Inputs<IrisInput>) -> Self::ProgramType {
        /*
         *        const N_REGISTERS: usize = 4;
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
        todo!()
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

    /*
     */
}

#[derive(Debug, Clone, Eq, PartialEq, FromPrimitive)]
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

impl RegisterRepresentable for IrisInput {
    type TrueType = IrisClass;

    fn argmax(registers: Registers) -> Option<Self::TrueType> {
        registers.argmax::<Self::TrueType>()
    }
}

impl Into<Registers> for IrisInput {
    fn into(self) -> Registers {
        return Registers(vec![
            self.sepal_length,
            self.sepal_width,
            self.petal_length,
            self.petal_width,
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
        let Inputs(inputs) = <TestLGP as Runnable>::load_inputs(tmpfile.path());
        assert_ne!(inputs.len(), 0);
        Ok(())
    }
}
