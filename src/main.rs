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
#[derive(Debug, Clone)]
struct Collection<ItemType>(Vec<ItemType>);

type Registers = Collection<f32>;

#[derive(Debug, Clone)]
struct Inputs<InputType: ArrayConvertable>(Collection<InputType>);

trait Verifiable: PartialEq + Eq + Debug {}

trait ArrayConvertable: Clone + fmt::Debug + Into<Registers>
where
    Self::TrueType: Verifiable,
{
    type TrueType;

    fn output_is_correct(&self, output_value: Self::TrueType) -> bool;
}

#[derive(Debug, Clone)]
enum Exemplars<'a, InputType> {
    Register(&'a Registers),
    Input(&'a Collection<InputType>),
}

trait Operable<InputType>: fmt::Debug
where
    InputType: ArrayConvertable,
{
    fn apply(
        &self,
        data_set: Exemplars<InputType>,
        registers: Registers,
        source: i8,
        target: i8,
    ) -> ();

    fn clone_dyn(&self) -> Box<dyn Operable<InputType>>;
}

impl<T: ArrayConvertable> Clone for Box<dyn Operable<T>> {
    fn clone(&self) -> Self {
        self.clone_dyn()
    }
}

#[derive(Clone, Debug)]
struct Instruction<'a, InputType> {
    source: i8,
    target: i8,
    mode: &'a Exemplars<'a, InputType>,
    registers: &'a Registers,
}

#[derive(Debug, Clone)]
struct Program<InputType>
where
    InputType: ArrayConvertable,
{
    instructions: Vec<Box<dyn Operable<InputType>>>,
    registers: Registers,
    inputs: Rc<Inputs<InputType>>,
}

trait Auditable {
    fn eval_fitness(&self) -> Fitness;
}

#[derive(Debug, Clone)]
struct HyperParameters {
    population_size: usize,
    n_generations: i8,
    selection_dropout: f32,
    input_file_path: PathBuf,
}

#[derive(Debug, Clone)]
struct Population<InputType: ArrayConvertable>(Collection<Program<InputType>>);

#[derive(Clone, Debug)]
struct LinearGeneticProgramming<InputType>
where
    InputType: ArrayConvertable,
{
    hyper_parameters: HyperParameters,
    population: Population<InputType>,
    inputs: Inputs<InputType>,
}

impl<InputType> LinearGeneticProgramming<InputType>
where
    InputType: ArrayConvertable,
{
    fn new<T>(lgp: T, hyper_parameters: HyperParameters) -> LinearGeneticProgramming<T::InputType>
    where
        T: Runnable,
        T::InputType: ArrayConvertable,
    {
        let inputs = lgp.load_inputs(&hyper_parameters.input_file_path);
        let population = lgp.init_population(hyper_parameters.population_size);

        return LinearGeneticProgramming {
            inputs,
            population,
            hyper_parameters,
        };
    }
}

struct Fitness(f32);

// TODO: Document usage.
struct TestLGP;

impl Runnable for TestLGP {
    type InputType = IrisInput;

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

    fn init_population(&self, size: usize) -> Population<Self::InputType> {
        todo!()
    }

    fn compete(&self, retention_percent: f32) -> Population<Self::InputType> {
        todo!()
    }

    fn run(&self, n_generations: usize) -> () {
        todo!()
    }
}

trait Runnable {
    type InputType: ArrayConvertable;

    fn load_inputs(&self, file_path: &Path) -> Inputs<Self::InputType>;
    fn init_population(&self, size: usize) -> Population<Self::InputType>;
    fn compete(&self, retention_percent: f32) -> Population<Self::InputType>;
    fn run(&self, n_generations: usize) -> ();
}

const IRIS_DATASET_LINK: &'static str =
    "https://archive.ics.uci.edu/ml/machine-learning-databases/iris/bezdekIris.data";

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

impl ArrayConvertable for IrisInput {
    type TrueType = IrisClass;

    fn output_is_correct(&self, output_value: Self::TrueType) -> bool {
        output_value == self.class
    }
}

impl Into<Registers> for IrisInput {
    fn into(self) -> Registers {
        return Collection(vec![
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
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use std::error;

    use tempfile::NamedTempFile;

    use super::*;

    #[tokio::test]
    async fn given_iris_dataset_when_csv_is_read_then_rows_are_deserialized_as_structs(
    ) -> Result<(), Box<dyn error::Error>> {
        let response = reqwest::get(IRIS_DATASET_LINK).await?;
        let content = response.text().await?;

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
        let test_lgp = TestLGP;
        let Inputs(Collection(inputs)) = Runnable::load_inputs(&test_lgp, tmpfile.path());
        assert_ne!(inputs.len(), 0);
        Ok(())
    }
}
