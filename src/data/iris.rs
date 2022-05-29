#[macro_export]
macro_rules! executable {
    ( $fn_name: ident,  $op: tt, $val: expr) => {
        fn $fn_name<'r>(registers: &'r mut [RegisterValue], data: &[RegisterValue]) -> &'r [RegisterValue] {
            assert_eq!(registers.len(), data.len());

            for index in 0..registers.len() {
                registers[index] = registers[index] $op $val
            }

            return registers;
        }
    };

    ( $fn_name: ident, $op: tt) => {
        fn $fn_name<'r>(registers: &'r mut [RegisterValue], data: &[RegisterValue]) -> &'r [RegisterValue] {
            assert_eq!(registers.len(), data.len());

            for index in 0..registers.len() {
                registers[index] = registers[index] $op data[index]
            }

            return registers;
        }
    };

}
mod iris_ops {
    use ordered_float::OrderedFloat;

    use crate::{genes::registers::RegisterValue, utils::alias::AnyExecutable};

    executable!(add, +);
    executable!(multiply, *);
    executable!(subtract, -);
    executable!(divide, /, OrderedFloat(2f64));

    pub const EXECUTABLES: &'static [AnyExecutable] =
        &[self::add, self::subtract, self::divide, self::multiply];
}

#[cfg(test)]
mod iris_tests {
    use std::error;

    use crate::{
        data::iris::iris_data::IrisInput,
        genes::{algorithm::HyperParameters, characteristics::FitnessScore},
        metrics::benchmarks::ComplexityBenchmark,
    };

    use super::iris_data::{IrisLinearGeneticProgramming, IRIS_DATASET_LINK};
    use more_asserts::{assert_le, assert_lt};
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
            executables: todo!(),
        };

        let inputs = IrisLinearGeneticProgramming::load_inputs(tmp_file.path());
        let mut gp = IrisLinearGeneticProgramming::new(hyper_params, &inputs);

        gp.init_population().evaluate().rank();

        const PLOT_FILE_NAME: &'static str = "/tmp/tests/plots/given_lgp_instance_when_sufficient_iterations_have_been_used_then_population_contains_the_same_benchmark_fitness.png";

        let mut benchmarks: Vec<ComplexityBenchmark<Option<FitnessScore>>> = vec![];
        let mut generations = 0;

        loop {
            let benchmark = gp.get_benchmark_individuals();
            benchmarks.push(benchmark);
            let benchmark_ref = benchmarks.last().unwrap();

            gp.apply_selection().breed().evaluate().rank();

            if benchmark_ref.worst == benchmark_ref.median
                && benchmark_ref.median == benchmark_ref.best
            {
                break;
            } else {
                generations += 1;

                if generations > hyper_params.max_generations {
                    // TODO: Create concrete error type; SNAFU or Failure?
                    return Err("Generations exceeded expect convergence time.")?;
                }
            }
        }

        let root = BitMapBackend::new(PLOT_FILE_NAME, (1280, 720)).into_drawing_area();
        root.fill(&WHITE)?;
        let mut chart = ChartBuilder::on(&root)
            .caption("Fitness Over Generations", ("sans-serif", 50).into_font())
            .margin(5u32)
            .x_label_area_size(30u32)
            .y_label_area_size(30u32)
            .build_cartesian_2d(0..benchmarks.len(), 0f64..1f64)?;

        chart.configure_mesh().draw()?;

        chart
            .draw_series(LineSeries::new(
                (0..benchmarks.len())
                    .map(|x_i| (x_i, benchmarks.get(x_i).unwrap().best.unwrap().into_inner())),
                &RED,
            ))?
            .label("Best")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

        chart
            .draw_series(LineSeries::new(
                (0..benchmarks.len()).map(|x_i| {
                    (
                        x_i,
                        benchmarks.get(x_i).unwrap().median.unwrap().into_inner(),
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
                        benchmarks.get(x_i).unwrap().worst.unwrap().into_inner(),
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
            executables: todo!(),
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
            executables: todo!(),
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
            executables: todo!(),
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
        let inputs = IrisLinearGeneticProgramming::load_inputs(tmpfile.path());
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
    };
    use strum::EnumCount;

    use crate::{
        genes::{
            algorithm::{GeneticAlgorithm, HyperParameters},
            characteristics::{Fitness, FitnessScore},
            chromosomes::Instruction,
            program::Program,
            registers::Registers,
        },
        metrics::{accuracy::Accuracy, benchmarks::Benchmark},
        utils::{
            alias::{AnyExecutable, Inputs},
            random::GENERATOR,
        },
    };

    use super::{
        iris_data::{IrisClass, IrisInput, IrisLinearGeneticProgramming},
        iris_ops,
    };

    impl<'a> Benchmark for IrisLinearGeneticProgramming {
        type InputType = FitnessScore;

        fn get_worst(&self) -> Option<Self::InputType> {
            match self.population.first() {
                Some(&Program { fitness, .. }) => fitness,
                _ => None,
            }
        }

        fn get_median(&self) -> Option<Self::InputType> {
            match self.population.middle() {
                Some(&Program { fitness, .. }) => fitness,
                _ => None,
            }
        }

        fn get_best(&self) -> Option<Self::InputType> {
            match self.population.last() {
                Some(&Program { fitness, .. }) => fitness,
                _ => None,
            }
        }
    }

    impl<'a> Program<'a, IrisInput> {
        pub fn generate(
            inputs: &'a Inputs<IrisInput>,
            max_instructions: usize,
            executables: &[AnyExecutable],
        ) -> Self {
            let register_len = 4;
            let input_len = 4;
            let registers = Registers::new(register_len);

            let n_instructions = UniformInt::<usize>::new(0, max_instructions).sample(GENERATOR);

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
        fn retrieve_fitness(&self) -> FitnessScore {
            let inputs = self.inputs;

            let mut fitness = Accuracy::<Option<usize>>::new();

            for input in inputs {
                let mut registers = self.registers.clone();

                for instruction in &self.instructions {
                    let [source_data, target_data] = instruction.get_data(&registers, input);

                    instruction.apply(&mut registers, source_data, target_data);
                }

                let correct_index = input.class as usize;
                let registers_argmax = registers.argmax(IrisClass::COUNT, correct_index);

                fitness.observe([registers_argmax, Some(correct_index)]);

                registers.reset();
            }

            let fitness_score = fitness.calculate();

            fitness_score
        }

        fn lazy_retrieve_fitness(&mut self) -> () {
            todo!()
        }
    }
}

pub mod iris_data {
    use core::fmt;
    use std::fmt::Display;

    use serde::{Deserialize, Serialize};
    use strum::EnumCount;

    use crate::genes::registers::{RegisterValue, Registers};

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
        Hash,
    )]
    pub enum IrisClass {
        #[serde(rename = "Iris-setosa")]
        Setosa = 0,
        #[serde(rename = "Iris-versicolor")]
        Versicolour = 1,
        #[serde(rename = "Iris-virginica")]
        Virginica = 2,
    }

    pub struct IrisLinearGeneticProgramming;

    #[derive(Deserialize, Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Serialize, Hash)]
    pub struct IrisInput {
        sepal_length: RegisterValue,
        sepal_width: RegisterValue,
        petal_length: RegisterValue,
        petal_width: RegisterValue,
        pub class: IrisClass,
    }

    impl Display for IrisInput {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let serialized = toml::to_string(&self).unwrap();
            f.write_str(&serialized)
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
