struct Registers<V>([V]);

trait Operation<V> {
    fn apply_operation(registers: Registers<V>, source: V, target: V) -> ();
}

struct Instruction<V> {
    source: V,
    target: V,
}

struct Program<V, O>
where
    O: Operation<V>,
{
    instructions: Vec<O>,
    registers: [V],
}

fn main() {
    println!("Hello, world!");
}
