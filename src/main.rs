trait Operation<V> {
    fn apply_operation(source: V, target: V) -> V;
}

struct Instruction<V> {
    source: V,
    target: V,
}

struct Program<'a, V, O>
where
    O: Operation<V>,
{
    instructions: &'a [O],
    registers: &'a [V],
}

fn main() {
    println!("Hello, world!");
}
