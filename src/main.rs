use core::fmt;
use std::{
    fmt::Debug,
    path::{Path, PathBuf},
    rc::Rc,
    u8,
};

use csv::ReaderBuilder;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};

/// Lets describe the steps we're trying to execuet.
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
struct TestLGP;

const IRIS_DATASET_LINK: &'static str =
    "https://archive.ics.uci.edu/ml/machine-learning-databases/iris/bezdekIris.data";

#[derive(Debug, Clone)]
struct Collection<ItemType>(Vec<ItemType>);

#[derive(Debug, Clone)]
struct Registers(Collection<f32>);

#[derive(Debug, Clone)]
struct Inputs<InputType: VectorConvertable>(Collection<InputType>);

#[derive(Debug, Clone)]
struct Fitness(f32);

trait Verifiable: PartialEq + Eq + Debug {}
trait Auditable {
    fn eval_fitness(&self) -> Fitness;
}
trait VectorConvertable: Clone + fmt::Debug + Into<Registers>
where
    Self::TrueType: Verifiable,
{
    type TrueType;

    fn output_is_correct(&self, output_value: Self::TrueType) -> bool;
}

trait Operable: fmt::Debug
where
    Self::InputType: VectorConvertable,
{
    type InputType;

    fn exec(&self) -> ();

    // Accessors.
    fn get_source_index(&self) -> i8;
    fn get_target_index(&self) -> i8;
    fn get_registers(&self) -> Registers;
    fn get_data(&self) -> Exemplars<Self::InputType>;

    fn dyn_clone(&self) -> Box<dyn Operable<InputType = Self::InputType>>;
}

trait Executable: fmt::Debug + Auditable
where
    Self::InputType: VectorConvertable,
{
    type InputType;

    fn get_input(&self) -> Option<Self::InputType>;
    fn get_instructions(&self) -> Vec<Box<dyn Operable<InputType = Self::InputType>>>;
    fn dyn_clone(&self) -> Box<dyn Executable<InputType = Self::InputType>>;
}

trait Runnable
where
    Self::InputType: VectorConvertable,
    Self::ExecutableType: Executable,
{
    type InputType;
    type ExecutableType;

    fn load_inputs(&self, file_path: &Path) -> Inputs<Self::InputType>;
    fn generate_individual(&self) -> Self::ExecutableType;
    fn init_population(&self, size: usize) -> Population<Self::ExecutableType>;
    fn compete(&self, retention_percent: f32) -> Population<Self::ExecutableType>;
}

#[derive(Debug, Clone)]
enum Exemplars<'a, InputType>
where
    InputType: VectorConvertable,
{
    Register(&'a Registers),
    Input(&'a InputType),
}

impl<T: VectorConvertable> Clone for Box<dyn Operable<InputType = T>> {
    fn clone(&self) -> Self {
        self.dyn_clone()
    }
}

#[derive(Clone, Debug)]
struct Instruction<'a, InputType>
where
    InputType: VectorConvertable,
{
    source: i8,
    target: i8,
    mode: &'a Exemplars<'a, InputType>,
    registers: &'a Registers,
}

#[derive(Debug, Clone)]
struct Program<InputType>
where
    InputType: VectorConvertable,
{
    instructions: Vec<Box<dyn Operable<InputType = InputType>>>,
    inputs: Rc<Inputs<InputType>>,
}

impl Auditable for Program<IrisInput> {
    fn eval_fitness(&self) -> Fitness {
        /*
         *for Inputs(input in &self.inputs {
         *    let mut registers = Collection(Vec::new());
         *    for instruction in &self.instructions {
         *        instruction.apply(input, Registers(registers), 0, 1);
         *    }
         *}
         */

        return Fitness(0.);
    }
}

impl Executable for Program<IrisInput> {
    type InputType = IrisInput;

    fn get_input(&self) -> Option<Self::InputType> {
        todo!()
    }

    fn get_instructions(&self) -> Vec<Box<dyn Operable<InputType = Self::InputType>>> {
        todo!()
    }

    fn dyn_clone(&self) -> Box<dyn Executable<InputType = Self::InputType>> {
        todo!()
    }
}

#[derive(Debug, Clone)]
struct HyperParameters {
    population_size: usize,
    n_generations: i8,
    selection_dropout: f32,
    input_file_path: PathBuf,
}

impl<InputType> Clone for Box<dyn Executable<InputType = InputType>>
where
    InputType: VectorConvertable,
{
    fn clone(&self) -> Self {
        return self.dyn_clone();
    }
}

#[derive(Debug, Clone)]
struct Population<ExecutableType: Executable>(Collection<ExecutableType>);

#[derive(Debug, Clone)]
struct LinearGeneticProgramming<InputType, ExecutableType>
where
    InputType: VectorConvertable,
    ExecutableType: Executable,
{
    hyper_parameters: HyperParameters,
    population: Population<ExecutableType>,
    inputs: Inputs<InputType>,
}

impl<InputType, ExecutableType> LinearGeneticProgramming<InputType, ExecutableType>
where
    InputType: VectorConvertable,
    ExecutableType: Executable,
{
    fn new<T>(
        lgp: T,
        hyper_parameters: HyperParameters,
    ) -> LinearGeneticProgramming<T::InputType, T::ExecutableType>
    where
        T: Runnable,
        T::InputType: VectorConvertable,
    {
        let inputs = lgp.load_inputs(&hyper_parameters.input_file_path);
        let population = lgp.init_population(hyper_parameters.population_size);

        return LinearGeneticProgramming {
            inputs,
            population,
            hyper_parameters,
        };
    }

    fn run(&self, lgp: impl Runnable) {}
}

impl Runnable for TestLGP {
    type InputType = IrisInput;
    type ExecutableType = Program<Self::InputType>;

    fn load_inputs(&self, file_path: &Path) -> Inputs<Self::InputType> {
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(false)
            .from_path(file_path)
            .unwrap();

        let raw_inputs: Vec<IrisInput> = csv_reader
            .deserialize()
            .map(|input| -> Self::InputType { input.unwrap() })
            .collect();

        let inputs = Collection(raw_inputs);

        return Inputs(inputs);
    }

    fn init_population(&self, size: usize) -> Population<Self::ExecutableType> {
        todo!()
    }

    fn compete(&self, retention_percent: f32) -> Population<Self::ExecutableType> {
        todo!()
    }

    fn generate_individual(&self) -> Self::ExecutableType {
        todo!()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum IrisClass {
    Setosa,
    Versicolour,
    Virginica = 2,
}

impl Verifiable for IrisClass {}

#[derive(Deserialize, Debug, Clone)]
struct IrisInput {
    sepal_length: f32,
    sepal_width: f32,
    petal_length: f32,
    petal_width: f32,
    #[serde(deserialize_with = "IrisInput::deserialize_iris_class")]
    class: IrisClass,
}

impl VectorConvertable for IrisInput {
    type TrueType = IrisClass;

    fn output_is_correct(&self, output_value: Self::TrueType) -> bool {
        output_value == self.class
    }
}

impl Into<Registers> for IrisInput {
    fn into(self) -> Registers {
        return Registers(Collection(vec![
            self.sepal_length,
            self.sepal_width,
            self.petal_length,
            self.petal_width,
        ]));
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
        let test_lgp = TestLGP;
        let Inputs(Collection(inputs)) = Runnable::load_inputs(&test_lgp, tmpfile.path());
        assert_ne!(inputs.len(), 0);
        Ok(())
    }
}
