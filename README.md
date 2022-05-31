# Linear Genetic Programming

A framework for implementing algorithms involving Linear Genetic Programming.

![build passing](https://github.com/urmzd/linear-genetic-programming/actions/workflows/develop.yml/badge.svg)

## Modules

- [Metrics](src/metrics)
- [Data](src/data)
- [Genes](src/genes)
- [Utils](src/utils)

## Examples

### Iris Dataset Implementation

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let ContentFilePair(_, file) = get_iris_content().await?;
    let inputs = IrisLgp::load_inputs(file.path());

    let hyper_params: HyperParameters<Program<IrisInput>> = HyperParameters {
        population_size: 1000,
        gap: 0.5,
        max_generations: 5,
        program_params: ProgramGenerateParams {
            inputs: &inputs,
            max_instructions: 100,
            executables: IRIS_EXECUTABLES,
        },
    };

    IrisLgp::execute(&hyper_params);
    Ok(())
}
```

## Building

Requirements:
- Cargo
- Stable Rust 

## Contributions

Contributions are welcomed. The guidelines can be found in [CONTRIBUTING.md](./CONTRIBUTING.md).
