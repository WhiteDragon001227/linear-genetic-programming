# Linear Genetic Programming

A framework for implementing algorithms involving Linear Genetic Programming.

## Modules

-   [Metrics](src/metrics.rs)
-   [Iris](src/iris.rs)
-   [Algorithm](src/algorithm.rs)

## Examples

### Iris Dataset Implementation

```rust
  fn main() {
    let hyper_params = HyperParameters {
        population_size: 1000,
        max_program_size: 100,
        gap: 0.5,
        max_generations: 100,
    };

    let inputs = IrisLinearGeneticProgramming::load_inputs("./data.csv");
    let mut gp = IrisLinearGeneticProgramming::new(hyper_params, &inputs);

    gp.init_population().eval_population();

    for _ in 0..hyper_params.max_generations {
        gp.apply_selection().breed();
    }

    println!(
        "Best Fitness: {}",
        gp.population.last().unwrap().fitness.unwrap()
    );
  }
```
