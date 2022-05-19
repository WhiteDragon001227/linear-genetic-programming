use std::{marker::PhantomData, path::Path};

use csv::ReaderBuilder;
use linear_genetic_programming::{
    algorithm::{GeneticAlgorithm, Population},
    fitness::Fitness,
    inputs::Inputs,
    iris::iris_data::IrisInput,
    program::Program,
};
use rand::prelude::SliceRandom;

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

impl<'a> GeneticAlgorithm<'a> for TestLGP<'a> {
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

    fn init_population(
        size: usize,
        max_instructions: usize,
        inputs: &'a Inputs<Self::InputType>,
    ) -> Population<'a, Self::InputType> {
        (0..size)
            .map(|_| Program::generate(inputs, max_instructions))
            .collect()
    }

    fn retrieve_selection(
        population: Population<'a, Self::InputType>,
        retention_rate: f32,
    ) -> Population<'a, Self::InputType> {
        assert!(retention_rate >= 0f32 && retention_rate <= 1f32);

        let mut sorted_population = population.clone();
        sorted_population.sort_by_cached_key(|p| p.eval_fitness());

        let lowest_index =
            ((1f32 - retention_rate) * (sorted_population.len() as f32)).floor() as i32 as usize;

        let keep_pop = &sorted_population[lowest_index..];
        let mut new_pop = Vec::with_capacity(population.capacity());

        for el in keep_pop.iter() {
            new_pop.push(el.clone())
        }

        new_pop
    }

    fn breed(population: Population<'a, Self::InputType>) -> Population<'a, Self::InputType> {
        let remaining_size = population.capacity() - population.len();

        let mut new_population = population.clone();

        let new_individuals = population.choose_multiple(&mut rand::thread_rng(), remaining_size);
        for individual in new_individuals {
            new_population.push(individual.clone())
        }

        new_population
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

    use linear_genetic_programming::iris::iris_data::IRIS_DATASET_LINK;

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
    async fn given_population_and_retention_rate_when_selection_occurs_then_population_is_cut_by_dropout(
    ) -> Result<(), Box<dyn error::Error>> {
        let ContentFilePair(_, tmp_file) = get_iris_content().await?;
        let inputs = <TestLGP as GeneticAlgorithm>::load_inputs(tmp_file.path());

        const SIZE: usize = 100;
        const MAX_INSTRUCTIONS: usize = 100;

        let population =
            <TestLGP as GeneticAlgorithm>::init_population(SIZE, MAX_INSTRUCTIONS, &inputs);

        const RETENTION_RATE: f32 = 0.5;

        let selected_population =
            <TestLGP as GeneticAlgorithm>::retrieve_selection(population, RETENTION_RATE);

        println!("{}", selected_population.len());

        assert!(
            selected_population.len()
                == ((SIZE as f32 * (1f32 - RETENTION_RATE)).floor() as i32 as usize)
        );

        Ok(())
    }

    #[tokio::test]
    async fn given_inputs_and_hyperparams_when_population_is_initialized_then_population_generated_with_hyperparams_and_inputs(
    ) -> Result<(), Box<dyn error::Error>> {
        let ContentFilePair(_, tmp_file) = get_iris_content().await?;
        let inputs = <TestLGP as GeneticAlgorithm>::load_inputs(tmp_file.path());
        const SIZE: usize = 100;
        const MAX_INSTRUCTIONS: usize = 100;
        let population =
            <TestLGP as GeneticAlgorithm>::init_population(SIZE, MAX_INSTRUCTIONS, &inputs);

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
        let inputs = <TestLGP as GeneticAlgorithm>::load_inputs(tmpfile.path());
        assert_ne!(inputs.len(), 0);
        Ok(())
    }
}
