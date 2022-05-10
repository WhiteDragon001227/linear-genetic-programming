use core::fmt;
use std::{path::Path, rc::Rc, u8};

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
///     2. Generate programs (instructions, registers, etc..) -- Init Popuulation
///     3. Eval Fitness
///     4. Drop x%
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
type Inputs<InputType> = Collection<InputType>;

trait InputTypeAttr: Clone + fmt::Debug + Into<Registers> {
    type TrueType;

    fn output_is_correct(&self, output_value: Self::TrueType) -> bool;
}

#[derive(Debug, Clone)]
enum Exemplars<'a, InputType> {
    Register(&'a Registers),
    Input(&'a Collection<InputType>),
}

trait Operation<InputType>: fmt::Debug
where
    InputType: InputTypeAttr,
{
    fn apply(
        &self,
        data_set: Exemplars<InputType>,
        registers: Registers,
        source: i8,
        target: i8,
    ) -> ();

    fn clone_dyn(&self) -> Box<dyn Operation<InputType>>;
}

impl<T: InputTypeAttr> Clone for Box<dyn Operation<T>> {
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
    InputType: InputTypeAttr,
{
    instructions: Vec<Box<dyn Operation<InputType>>>,
    registers: Registers,
    inputs: Rc<Inputs<InputType>>,
}

#[derive(Debug, Clone)]
struct HyperParameters {
    population_size: usize,
    n_generations: i8,
    selection_dropout: f32,
}

#[derive(Clone, Debug)]
struct LinearGeneticProgramming<InputType>
where
    InputType: InputTypeAttr,
{
    hyper_parameters: HyperParameters,
    population: Collection<Program<InputType>>,
    inputs: Inputs<InputType>,
}

struct Fitness(f32);

impl GeneticProgramming for LinearGeneticProgramming<IrisInput> {
    type InputType = IrisInput;

    fn load_inputs(&self, file_path: &Path) -> LinearGeneticProgramming<Self::InputType> {
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(false)
            .from_path(file_path)
            .unwrap();

        let raw_inputs: Vec<IrisInput> = csv_reader
            .deserialize()
            .map(|input| -> Self::InputType { input.unwrap() })
            .collect();

        let inputs = Collection(raw_inputs);

        return LinearGeneticProgramming {
            inputs,
            ..self.clone()
        };
    }

    fn init(
        &self,
        hyper_parameters: HyperParameters,
        inputs: Collection<Self::InputType>,
    ) -> LinearGeneticProgramming<Self::InputType> {
        todo!()
    }

    fn eval_fitness(&self) -> LinearGeneticProgramming<Self::InputType> {
        todo!()
    }

    fn compete(&self, percentage: f32) -> LinearGeneticProgramming<Self::InputType> {
        todo!()
    }

    fn run(&self) -> LinearGeneticProgramming<Self::InputType> {
        todo!()
    }
}

trait GeneticProgramming {
    type InputType: InputTypeAttr;

    fn load_inputs(&self, file_path: &Path) -> LinearGeneticProgramming<Self::InputType>;
    fn init(
        &self,
        hyper_parameters: HyperParameters,
        inputs: Collection<Self::InputType>,
    ) -> LinearGeneticProgramming<Self::InputType>;
    fn eval_fitness(&self) -> LinearGeneticProgramming<Self::InputType>;
    fn compete(&self, percentage: f32) -> LinearGeneticProgramming<Self::InputType>;
    fn run(&self) -> LinearGeneticProgramming<Self::InputType>;
}

const IRIS_DATASET_LINK: &'static str =
    "https://archive.ics.uci.edu/ml/machine-learning-databases/iris/bezdekIris.data";

#[derive(Debug, Clone, Eq, PartialEq)]
enum IrisClass {
    Setosa,
    Versicolour,
    Virginica = 2,
}

#[derive(Deserialize, Debug, Clone)]
struct IrisInput {
    sepal_length: f32,
    sepal_width: f32,
    petal_length: f32,
    petal_width: f32,
    #[serde(deserialize_with = "IrisInput::deserialize_iris_class")]
    class: IrisClass,
}

impl InputTypeAttr for IrisInput {
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

        for result in data {
            let record: IrisInput = result?;
            println!("{:?}", record)
        }

        Ok(())
    }

    #[tokio::test]
    async fn given_iris_dataset_when_csv_path_is_provided_then_collection_of_iris_structs_are_returned(
    ) -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }
}
