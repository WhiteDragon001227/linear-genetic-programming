use std::path::Path;

use lgp::{
    algorithm::{GeneticAlgorithm, HyperParameters},
    iris::iris_data::IrisLinearGeneticProgramming,
};

fn main() {
    let hyper_params = HyperParameters::new(1000, 100, 0.5);
    let path = Path::new("random_file.path");
    let inputs = IrisLinearGeneticProgramming::load_inputs(path);

    println!("Hello");
}
