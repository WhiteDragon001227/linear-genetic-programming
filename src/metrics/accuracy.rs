use std::{collections::HashMap, marker::PhantomData};

use ordered_float::OrderedFloat;

use crate::characteristics::FitnessScore;

use super::Metric;

struct RunningCounter(usize, usize);
impl RunningCounter {
    pub fn increment(&mut self) -> () {
        &self.0 += 1;
        &self.1 += 1;
    }

    pub fn increment_total(&mut self) -> () {
        &self.1 += 1;
    }

    pub fn get_total(&self) -> usize {
        return self.1;
    }

    pub fn get_counts(&self) -> usize {
        return self.0;
    }
}
struct OccuranceCounter<T, E>(T, RunningCounter, PhantomData<E>);

impl<T, E> OccuranceCounter<E> {
    fn new(expected_value: E) -> Self {
        OccuranceCounter(expected_value, RunningCounter(0, 0))
    }
}

impl<T, E> Metric for OccuranceCounter<T, E>
where
    T: PartialEq<E>,
{
    type ObservableType = T;
    type ResultType = RunningCounter;

    fn observe(&mut self, value: Self::ObservableType) -> () {
        let OccuranceCounter(expected_value, mut counter) = self;
        if value == expected_value {
            counter.increment()
        } else {
            counter.increment_total()
        }
    }

    fn calculate(&self) -> Self::ResultType {
        todo!()
    }
}

pub struct Accuracy<T>(HashMap<String, OccuranceCounter<T>>);

impl<T> Accuracy<T> {
    fn new() -> Self {
        Accuracy(HashMap::new())
    }
}

impl<T> Metric for Accuracy<T> {
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
        let Accuracy(n_correct, total) = self;
        OrderedFloat(*n_correct as f32) / OrderedFloat(*total as f32)
    }
}
