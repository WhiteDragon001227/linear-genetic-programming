use std::{cmp::Reverse, collections::BinaryHeap};

use crate::utils::Compare;

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
            self.max_heap.push(value);
        } else {
            self.min_heap.push(Reverse(value));
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
