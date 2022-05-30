use std::error;

use lgp::{
    data::iris::{data::IrisInput, ops::IRIS_EXECUTABLES},
    genes::{
        algorithm::{GeneticAlgorithm, HyperParameters, Loader},
        characteristics::FitnessScore,
        program::{Program, ProgramGenerateParams},
    },
    metrics::benchmarks::{Benchmark, ComplexityBenchmark},
};

use lgp::data::iris::{
    data::IrisLgp,
    utils::{get_iris_content, ContentFilePair},
};
use more_asserts::{assert_le, assert_lt};
use plotters::{
    prelude::{BitMapBackend, ChartBuilder, IntoDrawingArea, LineSeries, PathElement},
    style::{Color, IntoFont, BLACK, BLUE, GREEN, RED, WHITE},
};
use pretty_assertions::{assert_eq, assert_ne};

#[tokio::test]
async fn given_lgp_instance_when_sufficient_iterations_have_been_used_then_population_contains_the_same_benchmark_fitness(
) -> Result<(), Box<dyn error::Error>> {
    IrisLgp::init_env();

    let ContentFilePair(_, tmp_file) = get_iris_content().await?;
    let inputs = IrisLgp::load_inputs(tmp_file.path());

    let hyper_params: HyperParameters<Program<IrisInput>> = HyperParameters {
        population_size: 100,
        gap: 0.5,
        max_generations: 100,
        program_params: ProgramGenerateParams {
            max_instructions: 100,
            inputs: &inputs,
            executables: IRIS_EXECUTABLES,
        },
    };

    let mut population = IrisLgp::init_population(&hyper_params);
    IrisLgp::evaluate(&mut population);
    IrisLgp::rank(&mut population);

    const PLOT_FILE_NAME: &'static str = "/tmp/tests/plots/given_lgp_instance_when_sufficient_iterations_have_been_used_then_population_contains_the_same_benchmark_fitness.png";

    let mut benchmarks: Vec<ComplexityBenchmark<Option<FitnessScore>>> = vec![];
    let mut generations = 0;

    loop {
        let benchmark = population.get_benchmark_individuals();
        benchmarks.push(benchmark);
        let benchmark_ref = benchmarks.last().unwrap();

        IrisLgp::apply_selection(&mut population, hyper_params.gap);
        IrisLgp::breed(&mut population);
        IrisLgp::evaluate(&mut population);
        IrisLgp::rank(&mut population);

        if benchmark_ref.worst == benchmark_ref.median && benchmark_ref.median == benchmark_ref.best
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
        .build_cartesian_2d(0..benchmarks.len(), 0f32..1f32)?;

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

#[tokio::test]
async fn given_population_when_breeding_occurs_then_population_capacity_is_met(
) -> Result<(), Box<dyn error::Error>> {
    let ContentFilePair(_, tmp_file) = get_iris_content().await?;

    let inputs = IrisLgp::load_inputs(tmp_file.path());

    let hyper_params: HyperParameters<Program<IrisInput>> = HyperParameters {
        population_size: 100,
        gap: 0.5,
        max_generations: 100,
        program_params: ProgramGenerateParams {
            max_instructions: 100,
            inputs: &inputs,
            executables: IRIS_EXECUTABLES,
        },
    };

    let mut population = IrisLgp::init_population(&hyper_params);

    IrisLgp::apply_selection(&mut population, hyper_params.gap);

    let dropped_pop_len = population.len();

    assert_lt!(dropped_pop_len, hyper_params.population_size);

    IrisLgp::breed(&mut population);

    assert_eq!(population.len(), hyper_params.population_size);

    Ok(())
}

#[tokio::test]
async fn given_population_and_retention_rate_when_selection_occurs_then_population_is_cut_by_dropout(
) -> Result<(), Box<dyn error::Error>> {
    let ContentFilePair(_, tmp_file) = get_iris_content().await?;

    let inputs = IrisLgp::load_inputs(tmp_file.path());

    let hyper_params: HyperParameters<Program<IrisInput>> = HyperParameters {
        population_size: 100,
        gap: 0.5,
        max_generations: 100,
        program_params: ProgramGenerateParams {
            max_instructions: 100,
            inputs: &inputs,
            executables: IRIS_EXECUTABLES,
        },
    };

    let mut population = IrisLgp::init_population(&hyper_params);

    IrisLgp::apply_selection(&mut population, hyper_params.gap);

    self::assert_eq!(
        population.len(),
        ((hyper_params.population_size as f32 * (1f32 - hyper_params.gap)).floor() as i32 as usize)
    );

    Ok(())
}

#[tokio::test]
async fn given_inputs_and_hyperparams_when_population_is_initialized_then_population_generated_with_hyperparams_and_inputs(
) -> Result<(), Box<dyn error::Error>> {
    let ContentFilePair(_, tmp_file) = get_iris_content().await?;

    let inputs = IrisLgp::load_inputs(tmp_file.path());

    let hyper_params: HyperParameters<Program<IrisInput>> = HyperParameters {
        population_size: 100,
        gap: 0.5,
        max_generations: 100,
        program_params: ProgramGenerateParams {
            max_instructions: 100,
            inputs: &inputs,
            executables: IRIS_EXECUTABLES,
        },
    };

    let population = IrisLgp::init_population(&hyper_params);

    self::assert_eq!(population.len(), hyper_params.population_size);

    for individual in population.get_pop() {
        assert_le!(
            individual.instructions.len(),
            hyper_params.program_params.max_instructions
        )
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
    let inputs = IrisLgp::load_inputs(tmpfile.path());
    assert_ne!(inputs.len(), 0);
    Ok(())
}
