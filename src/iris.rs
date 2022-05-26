mod iris_ops {
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
    use std::error;

    use crate::{
        algorithm::{GeneticAlgorithm, HyperParameters, LinearGeneticProgramming},
        iris::iris_data::IrisInput,
        metrics::ComplexityBenchmark,
    };

    use super::iris_data::{IrisLinearGeneticProgramming, IRIS_DATASET_LINK};
    use more_asserts::{assert_le, assert_lt};
    use ordered_float::OrderedFloat;
    use plotters::{
        prelude::{BitMapBackend, ChartBuilder, IntoDrawingArea, LineSeries, PathElement},
        style::{Color, IntoFont, BLACK, BLUE, GREEN, RED, WHITE},
    };
    use pretty_assertions::{assert_eq, assert_ne};
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn given_lgp_instance_when_sufficient_iterations_have_been_used_then_population_contains_the_same_benchmark_fitness(
    ) -> Result<(), Box<dyn error::Error>> {
        IrisLinearGeneticProgramming::init_env();

        let ContentFilePair(_, tmp_file) = get_iris_content().await?;

        let hyper_params = HyperParameters {
            population_size: 100,
            max_program_size: 100,
            gap: 0.5,
            max_generations: 100,
        };

        let inputs = IrisLinearGeneticProgramming::load_inputs(tmp_file.path());
        let mut gp = IrisLinearGeneticProgramming::new(hyper_params, &inputs);

        gp.init_population().eval_population();

        let mut benchmark = ComplexityBenchmark {
            worst: gp.population.first().unwrap().fitness.unwrap(),
            median: gp.population.middle().unwrap().fitness.unwrap(),
            best: gp.population.last().unwrap().fitness.unwrap(),
        };

        let mut benchmarks: Vec<ComplexityBenchmark<OrderedFloat<f32>>> = vec![benchmark];

        let mut generations = 0;

        const PLOT_FILE_NAME: &'static str = "/tmp/tests/plots/given_lgp_instance_when_sufficient_iterations_have_been_used_then_population_contains_the_same_benchmark_fitness.png";

        while benchmark.get_worst() != benchmark.get_median()
            || benchmark.get_median() != benchmark.get_best()
        {
            gp.apply_selection().breed().eval_population();

            benchmark = gp.get_benchmark_individuals();

            generations += 1;

            if generations > hyper_params.max_generations {
                // TODO: Create concrete error type; SNAFU or Failure?
                Err("Generations exceeded expect convergence time.")?;
            }
        }

        let root = BitMapBackend::new(PLOT_FILE_NAME, (1280, 720)).into_drawing_area();
        root.fill(&WHITE)?;
        let mut chart = ChartBuilder::on(&root)
            .caption("Fitness Over Generations", ("sans-serif", 50).into_font())
            .margin(5u32)
            .x_label_area_size(30u32)
            .y_label_area_size(30u32)
            .build_cartesian_2d(0..benchmarks.len(), 0f32..1f32)?;

        chart.configure_mesh().draw()?;

        chart
            .draw_series(LineSeries::new(
                (0..benchmarks.len()).map(|x_i| {
                    (
                        x_i,
                        benchmarks
                            .get(x_i)
                            .unwrap()
                            .best
                            .fitness
                            .unwrap()
                            .into_inner(),
                    )
                }),
                &RED,
            ))?
            .label("Best")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

        chart
            .draw_series(LineSeries::new(
                (0..benchmarks.len()).map(|x_i| {
                    (
                        x_i,
                        benchmarks
                            .get(x_i)
                            .unwrap()
                            .median
                            .fitness
                            .unwrap()
                            .into_inner(),
                    )
                }),
                &GREEN,
            ))?
            .label("Median")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &GREEN));

        chart
            .draw_series(LineSeries::new(
                (0..benchmarks.len()).map(|x_i| {
                    (
                        x_i,
                        benchmarks
                            .get(x_i)
                            .unwrap()
                            .worst
                            .fitness
                            .unwrap()
                            .into_inner(),
                    )
                }),
                &BLUE,
            ))?
            .label("Worst")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;

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

        let inputs = IrisLinearGeneticProgramming::load_inputs(tmp_file.path());

        let hyper_params = HyperParameters {
            population_size: 100,
            max_program_size: 100,
            gap: 0.5,
            max_generations: 100,
        };

        let mut gp = IrisLinearGeneticProgramming::new(hyper_params, &inputs);

        gp.init_population().apply_selection();

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

        let inputs = IrisLinearGeneticProgramming::load_inputs(tmp_file.path());

        let hyper_params = HyperParameters {
            population_size: 100,
            max_program_size: 100,
            gap: 0.5,
            max_generations: 100,
        };

        let mut gp = IrisLinearGeneticProgramming::new(hyper_params, &inputs);

        gp.init_population().apply_selection();

        self::assert_eq!(
            gp.population.len(),
            ((hyper_params.population_size as f32 * (1f32 - hyper_params.gap)).floor() as i32
                as usize)
        );

        Ok(())
    }

    #[tokio::test]
    async fn given_inputs_and_hyperparams_when_population_is_initialized_then_population_generated_with_hyperparams_and_inputs(
    ) -> Result<(), Box<dyn error::Error>> {
        let ContentFilePair(_, tmp_file) = get_iris_content().await?;

        let inputs = IrisLinearGeneticProgramming::load_inputs(tmp_file.path());

        let hyper_params = HyperParameters {
            population_size: 100,
            max_program_size: 100,
            gap: 0.5,
            max_generations: 100,
        };

        let mut gp = IrisLinearGeneticProgramming::new(hyper_params, &inputs);

        gp.init_population();

        self::assert_eq!(gp.population.len(), hyper_params.population_size);

        for individual in gp.population.get_pop() {
            assert_le!(individual.instructions.len(), hyper_params.max_program_size)
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
        let inputs = LinearGeneticProgramming::<IrisInput>::load_inputs(tmpfile.path());
        assert_ne!(inputs.len(), 0);
        Ok(())
    }
}

mod iris_impl {
    use std::{collections::VecDeque, path::PathBuf};

    use csv::ReaderBuilder;

    use more_asserts::assert_le;
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
        characteristics::{Fitness, FitnessScore},
        inputs::Inputs,
        instruction::Instruction,
        metrics::{Accuracy, Metric},
        program::Program,
        registers::{RegisterRepresentable, Registers},
    };

    use super::iris_data::{IrisClass, IrisInput, IrisLinearGeneticProgramming};

    impl<'a> GeneticAlgorithm<'a> for IrisLinearGeneticProgramming<'a> {
        type InputType = IrisInput;

        fn load_inputs(file_path: impl Into<PathBuf>) -> Inputs<Self::InputType> {
            let mut csv_reader = ReaderBuilder::new()
                .has_headers(false)
                .from_path(file_path.into())
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
                let program = Program::generate(&self.inputs, self.hyper_params.max_program_size);
                VecDeque::push_front(self.population.get_mut_pop(), program)
            }

            self
        }

        fn eval_population(&mut self) -> &mut Self {
            for individual in self.population.get_mut_pop() {
                individual.fitness = match individual.fitness {
                    None => Some(individual.eval_fitness()),
                    Some(fitness) => Some(fitness),
                }
            }

            self.population.sort();

            self
        }

        fn apply_selection(&mut self) -> &mut Self {
            let HyperParameters { gap, .. } = self.hyper_params;

            assert!(gap >= 0f32 && gap <= 1f32);

            assert_le!(
                self.population.first().unwrap().fitness,
                self.population.last().unwrap().fitness
            );

            let pop_len = self.population.len();

            let lowest_index = ((1f32 - gap) * (pop_len as f32)).floor() as i32 as usize;

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

            let mut fitness = Accuracy::new();

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
    use std::fmt::Display;

    use ordered_float::OrderedFloat;
    use serde::{Deserialize, Serialize};
    use strum::EnumCount;

    use crate::{
        algorithm::LinearGeneticProgramming,
        registers::{Compare, RegisterRepresentable, Registers, Show},
    };

    pub const IRIS_DATASET_LINK: &'static str =
        "https://archive.ics.uci.edu/ml/machine-learning-databases/iris/bezdekIris.data";

    #[derive(
        Debug,
        Clone,
        Copy,
        Eq,
        PartialEq,
        EnumCount,
        PartialOrd,
        Ord,
        strum::Display,
        Serialize,
        Deserialize,
    )]
    #[serde(rename_all = "lowercase")]
    pub enum IrisClass {
        Setosa = 0,
        Versicolour = 1,
        Virginica = 2,
    }

    pub type IrisLinearGeneticProgramming<'a> = LinearGeneticProgramming<'a, IrisInput>;

    #[derive(Deserialize, Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Serialize)]
    pub struct IrisInput {
        sepal_length: OrderedFloat<f32>,
        sepal_width: OrderedFloat<f32>,
        petal_length: OrderedFloat<f32>,
        petal_width: OrderedFloat<f32>,
        pub class: IrisClass,
    }

    impl Display for IrisInput {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let serialized = toml::to_string(&self).unwrap();
            f.write_str(&serialized)
        }
    }

    impl Show for IrisInput {}
    impl Compare for IrisInput {}

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
}
