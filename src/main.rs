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

#[derive(PartialEq, Eq, Debug)]
struct Dimensions(u32, Option<u32>);

struct Collection<ItemType>(Vec<ItemType>);

// Concrete types (for testing purposes).
struct Add;
struct Subtract;
struct Multiply;
struct Divide;

type InternalRegisters = Collection<f32>;

enum DataSet<InputType> {
    Register(InternalRegisters),
    Input(Collection<InputType>),
}

trait Operation {
    type InputType;

    fn apply(&self, data_set: DataSet<Self::InputType>, source: i8, target: i8) -> ();
}

struct Instruction<InputType> {
    source: i8,
    target: i8,
    mode: DataSet<InputType>,
}

struct Program<InputType> {
    instructions: Vec<Box<dyn Operation<InputType = InputType>>>,
    registers: InternalRegisters,
    inputs: Collection<Collection<InputType>>,
}

trait GeneticProgramming {
    fn init_population(pop_size: usize) -> ();
    fn eval_fitness() -> Vec<f32>;
    fn select_from_pop() -> ();
    fn drop_from_pop() -> ();
    fn breed() -> ();
    fn compete() -> ();
    fn run(n_generations: i8) -> ();
}

struct LinearGeneticProgramming {}

fn main() {
    println!("Hello, world!");
}
