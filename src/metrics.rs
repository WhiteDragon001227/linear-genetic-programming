use ordered_float::OrderedFloat;

use crate::fitness::FitnessScore;

pub trait Metric {
    type ObservableType;
    type ResultType;

    fn observe(&mut self, value: Self::ObservableType);
    fn calculate(&self) -> Self::ResultType;
}

// n_correct, total
pub struct MacroAccuracy(usize, usize);

impl MacroAccuracy {
    pub fn new(initial_correct: usize, total_counted: usize) -> Self {
        MacroAccuracy(initial_correct, total_counted)
    }
}

impl Metric for MacroAccuracy {
    type ObservableType = bool;
    type ResultType = FitnessScore;

    fn observe(&mut self, value: Self::ObservableType) {
        let count = match value {
            true => 1,
            _ => 0,
        };

        self.0 += count;
        self.1 += 1
    }

    fn calculate(&self) -> Self::ResultType {
        let MacroAccuracy(n_correct, total) = self;
        OrderedFloat(*n_correct as f32) / OrderedFloat(*total as f32)
    }
}

pub struct Benchmark<'a, P> {
    pub worst: &'a P,
    pub median: &'a P,
    pub best: &'a P,
}

impl<'a, InputType> Benchmark<'a, InputType> {
    pub fn new(worst: &'a InputType, median: &'a InputType, best: &'a InputType) -> Self {
        Benchmark {
            worst,
            median,
            best,
        }
    }
}

pub trait BenchmarkMetric<'a> {
    type InputType;

    fn get_benchmark_individuals(&'a self) -> Benchmark<Self::InputType>;
}
