use std::{collections::VecDeque, error, io::Write, path::Path, ptr};

use csv::ReaderBuilder;
use lgp::{
    algorithm::{GeneticAlgorithm, HyperParameters, Population},
    fitness::Fitness,
    inputs::Inputs,
    iris::iris_data::{IrisInput, IRIS_DATASET_LINK},
    program::Program,
    registers::RegisterRepresentable,
};
#[cfg(test)]
use pretty_assertions::assert_eq;
use rand::seq::IteratorRandom;
use tempfile::NamedTempFile;

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
struct BasicLGP<'a, InputType>
where
    InputType: RegisterRepresentable,
{
    population: Population<'a, InputType>,
    inputs: &'a Inputs<InputType>,
    hyper_params: HyperParameters,
}

// TODO: Optimize code (reduce cloning).
impl<'a> GeneticAlgorithm<'a> for BasicLGP<'a, IrisInput> {
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

    fn new(
        hyper_params: lgp::algorithm::HyperParameters,
        inputs: &'a Inputs<Self::InputType>,
    ) -> Self {
        let population: Population<'a, Self::InputType> =
            Population::new(hyper_params.population_size);

        BasicLGP {
            population,
            inputs,
            hyper_params,
        }
    }

    fn init_population(&mut self) -> &mut Self {
        for _ in 0..self.hyper_params.population_size {
            let program = Program::generate(&self.inputs, self.hyper_params.instruction_size);
            VecDeque::push_front(self.population.get_mut_pop(), program)
        }

        self
    }

    fn eval_population(&mut self) -> &mut Self {
        for individual in self.population.get_mut_pop() {
            let fitness = individual.eval_fitness();
            individual.fitness = Some(fitness);
        }

        self
    }

    fn apply_natural_selection(&mut self) -> &mut Self {
        let HyperParameters { retention_rate, .. } = self.hyper_params;

        assert!(retention_rate >= 0f32 && retention_rate <= 1f32);

        let pop_len = self.population.len();

        let lowest_index = ((1f32 - retention_rate) * (pop_len as f32)).floor() as i32 as usize;

        self.population.sort();

        for _ in 0..lowest_index {
            self.population.f_pop();
        }

        self
    }

    fn breed(&mut self) -> &mut Self {
        let Self { population, .. } = self;
        let pop_cap = population.capacity();
        let pop_len = population.len();
        let remaining_size = pop_cap - pop_len;

        let selected_individuals: Vec<Program<'a, Self::InputType>> = population
            .get_pop()
            .iter()
            .cloned()
            .choose_multiple(&mut rand::thread_rng(), remaining_size);

        for individual in selected_individuals {
            population.push(individual)
        }

        self
    }
}

async fn get_iris_content() -> Result<ContentFilePair, Box<dyn error::Error>> {
    let tmp_file = NamedTempFile::new()?;
    let response = reqwest::get(IRIS_DATASET_LINK).await?;
    let content = response.text().await?;
    writeln!(&tmp_file, "{}", &content)?;

    Ok(ContentFilePair(content, tmp_file))
}

struct ContentFilePair(String, NamedTempFile);

// Lo, Mid, Hi
struct Benchmark<'a, P>(&'a P, &'a P, &'a P);

trait BenchmarkMetric<'a>
where
    Self::InputType: RegisterRepresentable,
{
    type InputType;
    fn get_benchmark_individuals(&'a self) -> Benchmark<Program<'a, Self::InputType>>;
}

impl<'a, InputType> BenchmarkMetric<'a> for BasicLGP<'a, InputType>
where
    InputType: RegisterRepresentable,
{
    type InputType = InputType;

    fn get_benchmark_individuals(&'a self) -> Benchmark<Program<'a, Self::InputType>> {
        let pop = &self.population;
        let worst = pop.get(0);
        let median_index = math::round::floor(pop.len() as f64 / 2 as f64, 1) as usize;
        let median = pop.get(median_index);
        let best = pop.get(pop.len() - 1);

        Benchmark(worst.unwrap(), median.unwrap(), best.unwrap())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let ContentFilePair(_, tmp_file) = get_iris_content().await?;

    let hyper_params = HyperParameters {
        population_size: 100,
        instruction_size: 100,
        retention_rate: 0.5,
    };

    let inputs = <BasicLGP<IrisInput> as GeneticAlgorithm>::load_inputs(tmp_file.path());
    let mut gp = <BasicLGP<IrisInput> as GeneticAlgorithm>::new(hyper_params, &inputs);
    gp.init_population().eval_population();
    let Benchmark(mut worst, mut median, mut best) = gp.get_benchmark_individuals();

    let mut i = 0;
    while !ptr::eq(worst, median) && !ptr::eq(median, best) {
        println!("Iteration: {}", i);
        gp.apply_natural_selection().breed();

        // todo: ensure only lower indices are removed
        Benchmark(worst, median, best) = gp.get_benchmark_individuals();
        println!(
            "{:.5} {:.5} {:.5}",
            worst.fitness.unwrap(),
            median.fitness.unwrap(),
            best.fitness.unwrap()
        );
        i += 1;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::error;

    use more_asserts::{assert_le, assert_lt};

    use super::*;

    #[tokio::test]
    async fn given_population_when_breeding_occurs_then_population_capacity_is_met(
    ) -> Result<(), Box<dyn error::Error>> {
        let ContentFilePair(_, tmp_file) = get_iris_content().await?;
        let inputs = <BasicLGP<IrisInput> as GeneticAlgorithm>::load_inputs(tmp_file.path());
        let hyper_params = HyperParameters {
            population_size: 100,
            instruction_size: 100,
            retention_rate: 0.5,
        };
        let mut gp = <BasicLGP<IrisInput> as GeneticAlgorithm>::new(hyper_params, &inputs);
        gp.init_population().apply_natural_selection();

        let dropped_pop_len = gp.population.len();

        assert_lt!(dropped_pop_len, hyper_params.population_size);

        gp.breed();

        self::assert_eq!(gp.population.len(), hyper_params.population_size);

        Ok(())
    }

    #[tokio::test]
    async fn given_population_and_retention_rate_when_selection_occurs_then_population_is_cut_by_dropout(
    ) -> Result<(), Box<dyn error::Error>> {
        let ContentFilePair(_, tmp_file) = get_iris_content().await?;
        let inputs = <BasicLGP<IrisInput> as GeneticAlgorithm>::load_inputs(tmp_file.path());
        let hyper_params = HyperParameters {
            population_size: 100,
            instruction_size: 100,
            retention_rate: 0.5,
        };
        let mut gp = <BasicLGP<IrisInput> as GeneticAlgorithm>::new(hyper_params, &inputs);
        gp.init_population().apply_natural_selection();

        self::assert_eq!(
            gp.population.len(),
            ((hyper_params.population_size as f32 * (1f32 - hyper_params.retention_rate)).floor()
                as i32 as usize)
        );

        Ok(())
    }

    #[tokio::test]
    async fn given_inputs_and_hyperparams_when_population_is_initialized_then_population_generated_with_hyperparams_and_inputs(
    ) -> Result<(), Box<dyn error::Error>> {
        let ContentFilePair(_, tmp_file) = get_iris_content().await?;
        let inputs = <BasicLGP<IrisInput> as GeneticAlgorithm>::load_inputs(tmp_file.path());
        let hyper_params = HyperParameters {
            population_size: 100,
            instruction_size: 100,
            retention_rate: 0.5,
        };

        let mut gp = <BasicLGP<IrisInput> as GeneticAlgorithm>::new(hyper_params, &inputs);
        gp.init_population();

        self::assert_eq!(gp.population.len(), hyper_params.population_size);

        for individual in gp.population.get_pop() {
            assert_le!(individual.instructions.len(), hyper_params.instruction_size)
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
        let inputs = <BasicLGP<IrisInput> as GeneticAlgorithm>::load_inputs(tmpfile.path());
        assert_ne!(inputs.len(), 0);
        Ok(())
    }
}
