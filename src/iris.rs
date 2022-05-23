pub mod iris_ops {
    use ordered_float::OrderedFloat;

    use crate::containers::CollectionIndexPair;
    use crate::registers::RegisterValue;
    use crate::utils::AnyExecutable;

    fn add(registers: &CollectionIndexPair, data: &CollectionIndexPair) -> RegisterValue {
        registers.get_value() + data.get_value()
    }

    fn subtract(registers: &CollectionIndexPair, data: &CollectionIndexPair) -> RegisterValue {
        registers.get_value() - data.get_value()
    }

    fn divide(registers: &CollectionIndexPair, _data: &CollectionIndexPair) -> RegisterValue {
        registers.get_value() / OrderedFloat(2f32)
    }

    fn multiply(registers: &CollectionIndexPair, data: &CollectionIndexPair) -> RegisterValue {
        registers.get_value() * data.get_value()
    }

    pub const EXECUTABLES: &'static [AnyExecutable; 4] =
        &[self::add, self::subtract, self::divide, self::multiply];
}

#[cfg(test)]
mod iris_tests {
    use std::{error, ptr};

    use crate::{
        algorithm::{GeneticAlgorithm, HyperParameters, LinearGeneticProgramming},
        iris::iris_data::IrisInput,
        metrics::{Benchmark, BenchmarkMetric},
    };

    use super::iris_data::IRIS_DATASET_LINK;
    use more_asserts::{assert_le, assert_lt};
    use pretty_assertions::{assert_eq, assert_ne};
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_everything() -> Result<(), Box<dyn error::Error>> {
        let ContentFilePair(_, tmp_file) = get_iris_content().await?;

        let hyper_params = HyperParameters {
            population_size: 100,
            instruction_size: 100,
            retention_rate: 0.5,
        };

        let inputs =
            <LinearGeneticProgramming<IrisInput> as GeneticAlgorithm>::load_inputs(tmp_file.path());
        let mut gp =
            <LinearGeneticProgramming<IrisInput> as GeneticAlgorithm>::new(hyper_params, &inputs);
        gp.init_population().eval_population();
        let Benchmark(mut worst, mut median, mut best) = gp.get_benchmark_individuals();

        let mut i = 0;
        // TODO: Remove `iteration` condition.
        while (!ptr::eq(worst, median) && !ptr::eq(median, best)) || i > 1000 {
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

    async fn get_iris_content() -> Result<ContentFilePair, Box<dyn error::Error>> {
        let tmp_file = NamedTempFile::new()?;
        let response = reqwest::get(IRIS_DATASET_LINK).await?;
        let content = response.text().await?;
        writeln!(&tmp_file, "{}", &content)?;

        Ok(ContentFilePair(content, tmp_file))
    }

    struct ContentFilePair(String, NamedTempFile);

    #[tokio::test]
    async fn given_population_when_breeding_occurs_then_population_capacity_is_met(
    ) -> Result<(), Box<dyn error::Error>> {
        let ContentFilePair(_, tmp_file) = get_iris_content().await?;
        let inputs =
            <LinearGeneticProgramming<IrisInput> as GeneticAlgorithm>::load_inputs(tmp_file.path());
        let hyper_params = HyperParameters {
            population_size: 100,
            instruction_size: 100,
            retention_rate: 0.5,
        };
        let mut gp =
            <LinearGeneticProgramming<IrisInput> as GeneticAlgorithm>::new(hyper_params, &inputs);
        gp.init_population().apply_natural_selection();

        let dropped_pop_len = gp.population.len();

        assert_lt!(dropped_pop_len, hyper_params.population_size);

        gp.breed();

        assert_eq!(gp.population.len(), hyper_params.population_size);

        Ok(())
    }

    #[tokio::test]
    async fn given_population_and_retention_rate_when_selection_occurs_then_population_is_cut_by_dropout(
    ) -> Result<(), Box<dyn error::Error>> {
        let ContentFilePair(_, tmp_file) = get_iris_content().await?;
        let inputs =
            <LinearGeneticProgramming<IrisInput> as GeneticAlgorithm>::load_inputs(tmp_file.path());
        let hyper_params = HyperParameters {
            population_size: 100,
            instruction_size: 100,
            retention_rate: 0.5,
        };
        let mut gp =
            <LinearGeneticProgramming<IrisInput> as GeneticAlgorithm>::new(hyper_params, &inputs);
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
        let inputs =
            <LinearGeneticProgramming<IrisInput> as GeneticAlgorithm>::load_inputs(tmp_file.path());
        let hyper_params = HyperParameters {
            population_size: 100,
            instruction_size: 100,
            retention_rate: 0.5,
        };

        let mut gp =
            <LinearGeneticProgramming<IrisInput> as GeneticAlgorithm>::new(hyper_params, &inputs);
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
        let inputs =
            <LinearGeneticProgramming<IrisInput> as GeneticAlgorithm>::load_inputs(tmpfile.path());
        assert_ne!(inputs.len(), 0);
        Ok(())
    }
}

mod iris_impl {
    use std::{collections::VecDeque, path::Path};

    use csv::ReaderBuilder;
    use rand::{
        distributions::uniform::{UniformInt, UniformSampler},
        prelude::IteratorRandom,
        thread_rng,
    };
    use strum::EnumCount;

    use crate::{
        algorithm::{
            self, GeneticAlgorithm, HyperParameters, LinearGeneticProgramming, Population,
        },
        fitness::{Fitness, FitnessScore},
        inputs::Inputs,
        instruction::Instruction,
        metrics::{Accuracy, Metric},
        program::Program,
        registers::{RegisterRepresentable, Registers},
    };

    use super::iris_data::{IrisClass, IrisInput, IrisLinearGeneticProgramming};

    impl<'a> GeneticAlgorithm<'a> for IrisLinearGeneticProgramming<'a> {
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
            hyper_params: algorithm::HyperParameters,
            inputs: &'a Inputs<Self::InputType>,
        ) -> Self {
            let population: Population<'a, Self::InputType> =
                Population::new(hyper_params.population_size);

            LinearGeneticProgramming {
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

    impl<'a> Program<'a, IrisInput> {
        pub fn generate(inputs: &'a Inputs<IrisInput>, max_instructions: usize) -> Self {
            let register_len = <IrisInput as RegisterRepresentable>::get_number_classes();
            let registers = Registers::new(register_len);
            let input_len = <IrisInput as RegisterRepresentable>::get_number_features();

            let executables = super::iris_ops::EXECUTABLES;

            let n_instructions =
                UniformInt::<usize>::new(0, max_instructions).sample(&mut thread_rng());

            let instructions: Vec<Instruction> = (0..n_instructions)
                .map(|_| Instruction::generate(register_len, input_len, executables))
                .collect();

            Program {
                instructions,
                registers,
                inputs,
                fitness: None,
            }
        }
    }

    impl<'a> Fitness for Program<'a, IrisInput> {
        fn eval_fitness(&self) -> FitnessScore {
            let inputs = self.inputs;

            let mut fitness = Accuracy::new(0, 0);

            for input in inputs {
                let mut registers = self.registers.clone();

                for instruction in &self.instructions {
                    let [source_data, target_data] = instruction.get_data(&registers, input);

                    instruction.apply(&mut registers, source_data, target_data);
                }

                let correct_index = input.class as usize;
                let registers_argmax = registers.argmax(IrisClass::COUNT, correct_index);

                <Accuracy as Metric>::observe(
                    &mut fitness,
                    Some(correct_index) == registers_argmax,
                );

                registers.reset();
            }

            let fitness_score = fitness.calculate();

            fitness_score
        }
    }
}

pub mod iris_data {
    use core::fmt;

    use ordered_float::OrderedFloat;
    use serde::{
        de::{self, Visitor},
        Deserialize, Deserializer,
    };
    use strum::EnumCount;

    use crate::{
        algorithm::LinearGeneticProgramming,
        registers::{RegisterRepresentable, Registers},
    };

    pub const IRIS_DATASET_LINK: &'static str =
        "https://archive.ics.uci.edu/ml/machine-learning-databases/iris/bezdekIris.data";

    #[derive(Debug, Clone, Copy, Eq, PartialEq, EnumCount, PartialOrd, Ord)]
    pub enum IrisClass {
        Setosa = 0,
        Versicolour = 1,
        Virginica = 2,
    }

    pub type IrisLinearGeneticProgramming<'a> = LinearGeneticProgramming<'a, IrisInput>;

    #[derive(Deserialize, Debug, Clone, PartialEq, Ord, PartialOrd, Eq)]
    pub struct IrisInput {
        sepal_length: OrderedFloat<f32>,
        sepal_width: OrderedFloat<f32>,
        petal_length: OrderedFloat<f32>,
        petal_width: OrderedFloat<f32>,
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
            return Registers::from(vec![
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
}
