use std::error;

use lgp::{
    data::iris::{
        data::{IrisInput, IrisLgp},
        ops::IRIS_EXECUTABLES,
        utils::{get_iris_content, ContentFilePair},
    },
    genes::{
        algorithm::{GeneticAlgorithm, HyperParameters, Loader},
        individuals::{Program, ProgramGenerateParams},
    },
};

#[derive(Debug, Clone)]
struct T(pub usize, pub usize);

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let ContentFilePair(_, file) = get_iris_content().await?;
    let inputs = IrisLgp::load_inputs(file.path());

    let hyper_params: HyperParameters<Program<IrisInput>> = HyperParameters {
        population_size: 1000,
        gap: 0.5,
        max_generations: 5,
        program_params: ProgramGenerateParams {
            inputs: &inputs,
            max_instructions: 100,
            executables: IRIS_EXECUTABLES,
        },
    };

    IrisLgp::execute(&hyper_params);
    Ok(())
}
