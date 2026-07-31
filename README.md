# parzen

A focused Rust implementation of the Tree-structured Parzen Estimator (TPE) for categorical
Bayesian optimization.

```bash
cargo add parzen
```

```rust
use parzen::{Direction, GammaStrategy, Study, TpeSampler, TpeSamplerConfig, TpeSamplerDeps};

let sampler = TpeSampler::new(
    TpeSamplerDeps { gamma_strategy: GammaStrategy::Default },
    TpeSamplerConfig {
        seed: 42,
        n_startup_trials: 5,
        prior_weight: TpeSamplerConfig::DEFAULT_PRIOR_WEIGHT,
    },
);
let mut study = Study::new(Direction::Maximize, sampler);

for _ in 0..20 {
    let candidate = study.suggest_categorical("candidate", 5);
    study.complete_trial(if candidate == 2 { 1.0 } else { 0.1 });
}

assert!(study.best_trial().unwrap().value > 0.5);
```

`parzen` is deterministic for a fixed seed and has no dependency on a language-model or prompt
framework. It is used by [Typesayer](https://github.com/moderately-ai/typesayer) for MIPRO prompt
optimization.

MSRV: Rust 1.88. Licensed under either MIT or Apache-2.0 at your option.
