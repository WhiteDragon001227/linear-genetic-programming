use crate::{structures::MedianHeap, utils::Compare};

use super::Metric;

#[derive(Debug, Clone)]
pub struct RunningBenchmark<'a, InputType> {
    median_heap: MedianHeap<'a, InputType>,
}

impl<'a, InputType> RunningBenchmark<'a, InputType>
where
    InputType: Compare,
{
    fn new() -> Self {
        RunningBenchmark {
            median_heap: MedianHeap::<InputType>::new(),
        }
    }
}

impl<'a, InputType> Metric for RunningBenchmark<'a, InputType>
where
    InputType: Compare,
{
    type ObservableType = &'a InputType;

    type ResultType = ComplexityBenchmark<Option<Self::ObservableType>>;

    fn observe(&mut self, value: Self::ObservableType) -> () {
        self.median_heap.insert(value);
    }

    fn calculate(&self) -> Self::ResultType {
        ComplexityBenchmark {
            worst: self.median_heap.min(),
            median: self.median_heap.median(),
            best: self.median_heap.max(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ComplexityBenchmark<InputType>
where
    InputType: Clone,
{
    pub worst: InputType,
    pub median: InputType,
    pub best: InputType,
}

pub trait Benchmark
where
    Self::InputType: Clone,
{
    type InputType;

    fn get_worst(&self) -> Option<Self::InputType>;

    fn get_median(&self) -> Option<Self::InputType>;

    fn get_best(&self) -> Option<Self::InputType>;

    fn get_benchmark_individuals(&self) -> ComplexityBenchmark<Option<Self::InputType>> {
        ComplexityBenchmark {
            worst: self.get_worst(),
            median: self.get_median(),
            best: self.get_best(),
        }
    }
}
