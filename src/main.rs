use core::fmt;
use std::{path::Path, rc::Rc};

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
///     6. Repeat from 3 until best == median == worst
///
///
/// Notes:
///     Inputs should be referenced. (RC?)
///

#[derive(Debug, Clone)]
struct Collection<ItemType>(Vec<ItemType>);
type Collections<ItemType> = Collection<Collection<ItemType>>;

type Registers = Collection<f32>;

trait InputTypeAttr: Clone + fmt::Debug + Into<Registers> {}

impl InputTypeAttr for Registers {}

#[derive(Debug, Clone)]
enum Exemplars<'a, InputType> {
    Register(&'a Registers),
    Input(&'a Collection<InputType>),
}

trait Operation<InputType = Registers>: fmt::Debug
where
    InputType: InputTypeAttr,
{
    fn apply(
        &self,
        data_set: Exemplars<InputType>,
        registers: Registers,
        source: i8,
        target: i8,
    ) -> ();

    fn clone_dyn(&self) -> Box<dyn Operation<InputType>>;
}

impl<T: InputTypeAttr> Clone for Box<dyn Operation<T>> {
    fn clone(&self) -> Self {
        self.clone_dyn()
    }
}

#[derive(Clone, Debug)]
struct Instruction<'a, InputType> {
    source: i8,
    target: i8,
    mode: &'a Exemplars<'a, InputType>,
    registers: &'a Registers,
}

#[derive(Debug)]
struct Program<InputType>
where
    InputType: InputTypeAttr,
{
    instructions: Vec<Box<dyn Operation<InputType>>>,
    registers: Registers,
    inputs: Rc<Collections<InputType>>,
}

struct HyperParameters {
    population_size: usize,
    n_generations: i8,
    selection_dropout: f32,
}

struct LinearGeneticProgramming<InputType>
where
    InputType: InputTypeAttr,
{
    hyper_parameters: HyperParameters,
    population: Collection<Program<InputType>>,
    inputs: Collections<InputType>,
}

struct Fitness(f32);

trait GeneticProgramming {
    type InputType: InputTypeAttr;

    fn load_inputs(&self, file_path: &Path) -> Collections<Self::InputType>;
    fn init(
        &self,
        hyper_parameters: HyperParameters,
        inputs: Collection<Self::InputType>,
    ) -> LinearGeneticProgramming<Self::InputType>;
    fn eval_fitness(&self) -> Collection<Fitness>;
    fn compete(&self, percentage: f32) -> Collection<Program<Self::InputType>>;
    fn run(&self) -> ();
}

// GET https://archive.ics.uci.edu/ml/machine-learning-databases/iris/bezdekIris.data
enum IrisClass {
    Setosa,
    Versicolour,
    Virginica,
}

struct IrisInput {
    sepal_length: f32,
    sepal_width: f32,
    petal_length: f32,
    petal_width: f32,
    class: IrisClass,
}

fn main() {
    println!("Hello, world!");
}
