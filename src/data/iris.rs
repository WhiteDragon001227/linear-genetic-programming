mod iris_ops {
    use crate::genes::registers::RegisterValue;
    use ordered_float::OrderedFloat;

    use crate::{executable, utils::alias::AnyExecutable};

    executable!(add, +);
    executable!(multiply, *);
    executable!(subtract, -);
    executable!(divide, /, OrderedFloat(2f64));

    pub const IRIS_EXECUTABLES: &'static [AnyExecutable] = &[
        AnyExecutable::new("add", self::add),
        AnyExecutable::new("subtract", self::subtract),
        AnyExecutable::new("divide", self::divide),
        AnyExecutable::new("multiply", self::multiply),
    ];
}

#[cfg(test)]
mod iris_tests {
    use std::error;

    use crate::{
        data::iris::{iris_data::IrisInput, iris_ops::IRIS_EXECUTABLES},
        genes::{
            algorithm::{GeneticAlgorithm, HyperParameters},
            characteristics::FitnessScore,
        },
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
            executables: IRIS_EXECUTABLES,
            data_path: tmp_file.path(),
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
            data_path: todo!(),
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
            data_path: todo!(),
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
            executables: IRIS_EXECUTABLES,
            data_path: tmp_file.path(),
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

    use crate::genes::{characteristics::Organism, program::Program, registers::ValidInput};

    use super::iris_data::IrisInput;

    // impl<'a> Benchmark for IrisLinearGeneticProgramming {
    //     type InputType = FitnessScore;

    //     fn get_worst(&self) -> Option<Self::InputType> {
    //         match self.population.first() {
    //             Some(&Program { fitness, .. }) => fitness,
    //             _ => None,
    //         }
    //     }

    //     fn get_median(&self) -> Option<Self::InputType> {
    //         match self.population.middle() {
    //             Some(&Program { fitness, .. }) => fitness,
    //             _ => None,
    //         }
    //     }

    //     fn get_best(&self) -> Option<Self::InputType> {
    //         match self.population.last() {
    //             Some(&Program { fitness, .. }) => fitness,
    //             _ => None,
    //         }
    //     }
    // }

    // TODO: Make default implementation
}

pub mod iris_data {
    use core::fmt;
    use std::fmt::Display;

    use serde::{Deserialize, Serialize};
    use strum::EnumCount;

    use crate::genes::{
        algorithm::GeneticAlgorithm,
        registers::{RegisterValue, Registers, ValidInput},
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

    impl<'a> GeneticAlgorithm<'a> for IrisLinearGeneticProgramming {
        type InputType = IrisInput;
    }

    #[derive(Deserialize, Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Serialize, Hash)]
    pub struct IrisInput {
        sepal_length: RegisterValue,
        sepal_width: RegisterValue,
        petal_length: RegisterValue,
        petal_width: RegisterValue,
        class: IrisClass,
    }

    impl Display for IrisInput {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let serialized = toml::to_string(&self).unwrap();
            f.write_str(&serialized)
        }
    }

    impl ValidInput for IrisInput {
        const N_CLASSES: usize = 3;
        const N_FEATURES: usize = 4;

        fn get_class(&self) -> usize {
            self.class as usize
        }

        fn as_registers<'a>(&'a self) -> &'a Registers {
            return &Registers::from(vec![
                self.sepal_length,
                self.sepal_width,
                self.petal_length,
                self.petal_length,
            ]);
        }
    }
}
