use std::{cmp::Reverse, collections::BinaryHeap, fmt::Display};

use ordered_float::OrderedFloat;

use crate::{
    characteristics::FitnessScore,
    registers::{Compare, RegisterRepresentable},
};

pub trait Metric {
    type ObservableType;
    type ResultType;

    fn new() -> Self;
    fn observe(&mut self, value: Self::ObservableType) -> ();
    fn calculate(&self) -> Self::ResultType;
}

// n_correct, total
pub struct MacroAccuracy(usize, usize);

impl Metric for MacroAccuracy {
    type ObservableType = bool;
    type ResultType = FitnessScore;

    fn new() -> Self {
        MacroAccuracy(0, 0)
    }

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

#[derive(Debug, Clone)]
pub struct MedianHeap<'a, InputType> {
    max_heap: BinaryHeap<&'a InputType>,
    min_heap: BinaryHeap<Reverse<&'a InputType>>,
    max_element: Option<&'a InputType>,
    min_element: Option<&'a InputType>,
}

impl<'a, InputType> MedianHeap<'a, InputType>
where
    InputType: Compare,
{
    pub fn new() -> Self {
        MedianHeap {
            max_heap: BinaryHeap::new(),
            min_heap: BinaryHeap::new(),
            max_element: None,
            min_element: None,
        }
    }

    pub fn insert(&mut self, value: &'a InputType) -> () {
        let wrapped_value = Some(value);

        if wrapped_value > self.max_element {
            self.max_element = wrapped_value
        }

        if wrapped_value < self.min_element {
            self.min_element = Some(value)
        }

        if wrapped_value <= self.median() {
            &self.max_heap.push(value);
        } else {
            &self.min_heap.push(Reverse(value));
        }

        self.rebalance()
    }

    fn rebalance(&mut self) -> () {
        if &self.max_heap.len() - &self.min_heap.len() > 1 {
            let max_element = self.max_heap.pop();
            self.min_heap.push(Reverse(max_element.unwrap()))
        } else if (self.min_heap.len() as i32) - (self.max_heap.len() as i32) < -1 {
            let min_element = self.min_heap.pop();
            self.max_heap.push(min_element.unwrap().0)
        } else {
            ()
        }
    }

    pub fn median(&self) -> Option<&'a InputType> {
        if &self.max_heap.len() > &self.min_heap.len() {
            if let Some(&element) = self.max_heap.peek() {
                Some(element)
            } else {
                None
            }
        } else if &self.max_heap.len() < &self.min_heap.len() {
            if let Some(&Reverse(element)) = self.min_heap.peek() {
                Some(element)
            } else {
                None
            }
        } else {
            match self.max_heap.peek() {
                Some(&element) => Some(element),
                None => None,
            }
        }
    }

    pub fn max(&self) -> Option<&'a InputType> {
        self.max_element
    }

    pub fn min(&self) -> Option<&'a InputType> {
        self.min_element
    }
}

#[derive(Debug, Clone)]
pub struct Benchmark<'a, InputType> {
    median_heap: MedianHeap<'a, InputType>,
}

pub struct ComplexityBenchmark<'a, InputType> {
    worst: Option<&'a InputType>,
    median: Option<&'a InputType>,
    best: Option<&'a InputType>,
}

impl<'a, InputType> ComplexityBenchmark<'a, InputType> {
    pub fn get_worst(&self) -> Option<&'a InputType> {
        self.worst
    }

    pub fn get_median(&self) -> Option<&'a InputType> {
        self.median
    }

    pub fn get_best(&self) -> Option<&'a InputType> {
        self.best
    }
}

impl<'a, InputType> Metric for Benchmark<'a, InputType>
where
    InputType: Compare,
{
    fn new() -> Self {
        Benchmark {
            median_heap: MedianHeap::<InputType>::new(),
        }
    }

    type ObservableType = &'a InputType;

    type ResultType = ComplexityBenchmark<'a, InputType>;

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
