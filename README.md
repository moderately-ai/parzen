# parzen

A high-performance Rust implementation of the Tree-structured Parzen Estimator (TPE) for
categorical, floating-point, integer, conditional, and explicitly grouped Bayesian optimization.

```bash
cargo add parzen
```

```rust
use parzen::{
    CategoricalDistribution, Direction, Distribution, SearchSpace, Study,
    TpeSampler, TpeSamplerConfig,
};

fn main() -> Result<(), parzen::ParzenError> {
    let mut space = SearchSpace::new();
    space.add(
        "candidate",
        Distribution::Categorical(CategoricalDistribution::new(5)?),
    )?;
    let sampler = TpeSampler::new(TpeSamplerConfig::performance(42))?;
    let mut study = Study::new(Direction::Maximize, sampler, space)?;

    for _ in 0..20 {
        let candidate = study.suggest_categorical("candidate")?;
        study.complete_trial(if candidate == 2 { 1.0 } else { 0.1 })?;
    }
    assert!(study.best_value().is_some_and(|value| value > 0.5));
    Ok(())
}
```

## Numeric distributions

```rust
use parzen::{Distribution, FloatDistribution, IntDistribution, SearchSpace};

# fn main() -> Result<(), parzen::ParzenError> {
let mut space = SearchSpace::new();
space.add("learning_rate", Distribution::Float(FloatDistribution::log(1e-6, 1.0)?))?;
space.add("depth", Distribution::Int(IntDistribution::linear(1, 15)?.with_step(2)?))?;
# Ok(()) }
```

## Conditional and grouped parameters

Conditional parameters use categorical-parent conditions. Explicit groups use a trial-aligned
mixture of products: one mixture component is selected for the complete candidate vector, preserving
correlations between categorical, numeric, and mixed interacting parameters.

```rust
use parzen::{CategoricalDistribution, Condition, Distribution, IntDistribution, SearchSpace};

# fn main() -> Result<(), parzen::ParzenError> {
let mut space = SearchSpace::new();
let optimizer = space.add("optimizer", Distribution::Categorical(CategoricalDistribution::new(2)?))?;
let momentum = space.add("momentum", Distribution::Int(IntDistribution::linear(0, 10)?))?;
space.add_condition(momentum, Condition::CategoricalIn {
    parent: optimizer,
    choices: vec![1].into_boxed_slice(),
})?;
# Ok(()) }
```

`TpeSamplerConfig::performance(seed)` retains the exact best set plus bounded recent and
deterministically sampled bad history. Its estimator memory and update work therefore do not grow
with the number of completed trials. Raw trial history remains complete and grows linearly.
`TpeSamplerConfig::optuna_compatible(seed)` uses exact full history and Optuna-style trial weights
and kernels; it does not promise identical random sequences. Both presets use Optuna's
`min(ceil(0.1 * n), 25)` gamma strategy; the Hyperopt square-root strategy is available explicitly.

Integer and stepped-float priors are uniform over legal grid points. Their TPE likelihoods integrate
probability over each discrete cell instead of scoring a pre-quantized continuous point. Injected
stepped floats within four ULPs of a grid point are stored in canonical grid form.

The crate is deterministic for a fixed seed, configuration, search space, observation order, minor
crate version, and target architecture. Serde support is enabled by default, validates distributions
while deserializing, rejects unknown distribution fields, and can be disabled with
`default-features = false`.

See [MIGRATING-0.1-TO-0.2.md](MIGRATING-0.1-TO-0.2.md) for the v0.1 migration guide.

MSRV: Rust 1.89. Licensed under either MIT or Apache-2.0 at your option.
