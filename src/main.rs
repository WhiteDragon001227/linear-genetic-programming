use core::fmt;
use std::rc::Rc;

/// Lets describe the steps we're trying to execuet.
///
/// First we initialize a population of programs.
///
/// Programs consist of instructions.
///
/// Instructions consist of four things, a source index, a target index, an operation and a mode.
/// Each instruction is executed. The resulting registers are the the "outputs" of the program.
///
///
/// Data can be retrieved from two places; the registers or the inputs (external data, likely from
/// the fs).
///
/// Ex (Input -> (N, M)):
///
/// 0.1, 0.2, 0.3, 0.4, 0.5
/// 0.2, 0.3, 0.4, 0.5, 0.6
/// ...
///
/// Ex (Registers -> (N,)):
///
/// 0.1, 0.2, 0.3, 0.4, 0.5
///
/// As demonstrated above, inputs can expand in another dimension (in the above case, its the #. of
/// rows) as long as they share a dimension (in the above case, its the #. of columns)
///
/// Smoke Test Algorithm:
///     1. Load input data
///     2. Generate programs (instructions, registers, etc..) -- Init Popuulation
///     3. Eval Fitness
///     4. Drop x%
///     5. Clone 1 - x %
///     6. Repeat from 3
///
///
/// Notes:
///     Inputs should be referenced. (RC?)
///

#[derive(PartialEq, Eq, Debug)]
struct Dimensions(u32, Option<u32>);

#[derive(Debug, Clone)]
struct Collection<ItemType>(Vec<ItemType>);

type InternalRegisters = Collection<f32>;

#[derive(Debug, Clone)]
enum Exemplars<'a, InputType> {
    Register(&'a InternalRegisters),
    Input(&'a Collection<InputType>),
}

trait Operation<InputType = InternalRegisters>: fmt::Debug
where
    InputType: Clone + fmt::Debug,
{
    fn apply(
        &self,
        data_set: Exemplars<InputType>,
        registers: InternalRegisters,
        source: i8,
        target: i8,
    ) -> ();
}

#[derive(Clone, Debug)]
struct Instruction<'a, InputType> {
    source: i8,
    target: i8,
    mode: &'a Exemplars<'a, InputType>,
    registers: &'a InternalRegisters,
}

#[derive(Debug)]
struct Program<InputType>
where
    InputType: Clone + fmt::Debug,
{
    instructions: Vec<Box<dyn Operation<InputType>>>,
    registers: InternalRegisters,
    inputs: Rc<Collection<Collection<InputType>>>,
}

impl<'a, T> Clone for Program<T>
where
    T: Clone + fmt::Debug,
{
    fn clone(&self) -> Self {
        todo!()
    }
}

trait GeneticProgramming {
    fn init_population(pop_size: usize) -> ();
    fn eval_fitness() -> Vec<f32>;
    fn select_from_pop(percentage: f32) -> ();
    fn breed() -> ();
    fn compete() -> ();
    fn run(n_generations: i8) -> ();
}

struct LinearGeneticProgramming {}

fn main() {
    println!("Hello, world!");
}
