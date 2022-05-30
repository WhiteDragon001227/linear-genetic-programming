use std::{collections::HashMap, hash::Hash, marker::PhantomData};

use ordered_float::OrderedFloat;

use crate::utils::alias::Compare;

use super::definitions::Metric;

type ComparablePair<K> = [K; 2];

struct RunningCounter(usize, usize);

impl RunningCounter {
    pub fn count_correct(&mut self) -> () {
        self.0 += 1;
        self.1 += 1;
    }

    pub fn count_wrong(&mut self) -> () {
        self.1 += 1;
    }

    pub fn get_total(&self) -> usize {
        return self.1;
    }

    pub fn get_correct(&self) -> usize {
        return self.0;
    }
}
struct OccuranceCounter<T>(RunningCounter, PhantomData<T>);

impl<K> OccuranceCounter<K>
where
    K: Compare + Hash,
{
    fn new() -> Self {
        OccuranceCounter(RunningCounter(0, 0), PhantomData)
    }
}

impl<K> Metric for OccuranceCounter<K>
where
    K: Compare + Hash,
{
    type ObservableType = ComparablePair<K>;
    type ResultType = [usize; 2];

    fn observe(&mut self, value: Self::ObservableType) -> () {
        let OccuranceCounter(counter, ..) = self;

        match value {
            [x, y] if x == y => counter.count_correct(),
            _ => counter.count_wrong(),
        }
    }

    fn calculate(&self) -> Self::ResultType {
        [self.0.get_correct(), self.0.get_total()]
    }
}

pub struct Accuracy<K>(HashMap<K, OccuranceCounter<K>>)
where
    K: Compare + Hash;

impl<K> Accuracy<K>
where
    K: Compare + Hash,
{
    pub fn new() -> Self {
        Accuracy(HashMap::<K, OccuranceCounter<K>>::new())
    }
}

impl<K> Metric for Accuracy<K>
where
    K: Compare + Hash,
{
    type ObservableType = ComparablePair<K>;
    type ResultType = OrderedFloat<f64>;

    fn observe(&mut self, value: Self::ObservableType) {
        let Accuracy(map, ..) = self;
        let [.., expected] = value.clone();

        if map.contains_key(&expected) {
            let counter = map.get_mut(&expected).unwrap();
            counter.observe(value);
        } else {
            let mut counter = OccuranceCounter::new();
            counter.observe(value.clone());
            map.insert(expected, counter);
        }
    }

    fn calculate(&self) -> Self::ResultType {
        let counter = &self.0.iter().fold([0; 2], |accum, item| {
            [
                accum[0] + item.1.calculate()[0],
                accum[1] + item.1.calculate()[1],
            ]
        });

        OrderedFloat(counter[0] as f64) / OrderedFloat(counter[1] as f64)
    }
}
