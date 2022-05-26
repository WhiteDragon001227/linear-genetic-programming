use crate::{median_heap::MedianHeap, registers::Compare};

use super::Metric;

#[derive(Debug, Clone)]
pub struct RunningBenchmark<'a, InputType> {
    median_heap: MedianHeap<'a, InputType>,
}

impl<'a, InputType> Metric for RunningBenchmark<'a, InputType>
where
    InputType: Compare,
{
    fn new() -> Self {
        RunningBenchmark {
            median_heap: MedianHeap::<InputType>::new(),
        }
    }

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

pub struct ComplexityBenchmark<InputType> {
    worst: InputType,
    median: InputType,
    best: InputType,
}

impl<InputType> ComplexityBenchmark<InputType> {
    pub fn get_worst(&self) -> &InputType {
        &self.worst
    }

    pub fn get_median(&self) -> &InputType {
        &self.median
    }

    pub fn get_best(&self) -> &InputType {
        &self.best
    }
}
