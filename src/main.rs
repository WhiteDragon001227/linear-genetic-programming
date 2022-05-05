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

#[derive(PartialEq, Eq, Debug)]
struct Dimensions(u32, Option<u32>);

struct Datum<DatumType, const SIZE: usize>([DatumType; SIZE]);
struct Data<DatumType, const SIZE: usize>(Vec<Datum<DatumType, SIZE>>);

struct Add;
struct Subtract;
struct Multiply;

struct DataSet<InputType, RegisterType, const INPUT_SIZE: usize, const REGISTER_SIZE: usize> {
    registers: Datum(),
}

trait DataRetriever {}

//struct DataSet<InputType, RegisterType, > {
//inputs: Inputs<InputType, Size>,
//registers: Input<RegisterType,
//}

//trait Operation<T> {
//fn apply_operation(data: Data, source: i8, target: i8) -> ();
//}

//struct Instruction {
//source: i8,
//target: i8,
//mode: Mode,
//}

//struct Program<
//RegisterType,
//OperationType,
//InputType,
//const NO_INSTRUCTIONS: usize,
//const NO_REGISTERS: usize,
//const NO_INPUTS: usize,
//> where
//OperationType: Operation,
//{
//instructions: [OperationType; INSTRUCTION_COUNT],
//data: [Mode; { REGISTERS_COUNT + INPUT_SIZE }],
//}

//trait GeneticProgramming {
//fn init_population(pop_size: usize) -> ();
//fn eval_fitness() -> Vec<f32>;
//fn select_from_pop() -> ();
//fn drop_from_pop() -> ();
//fn breed() -> ();
//fn compete() -> ();
//fn run(n_generations: i8) -> ();
//}

//struct LinearGeneticProgramming {}

fn main() {
    println!("Hello, world!");
}
