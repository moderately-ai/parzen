# Migrating from Parzen 0.1 to 0.2

Parzen 0.2 changes the API to support validated numeric distributions, compact trial storage, and
grouped/conditional optimization.

## Construction

`TpeSamplerDeps` was removed. Build an explicit search space and use a configuration preset:

```rust
let mut space = SearchSpace::new();
space.add("x", Distribution::Categorical(CategoricalDistribution::new(5)?))?;
let sampler = TpeSampler::new(TpeSamplerConfig::performance(seed))?;
let study = Study::new(Direction::Maximize, sampler, space)?;
```

Use `.startup_trials(...)`, `.prior_weight(...)`, `.gamma(...)`, `.weights(...)`, `.history(...)`,
and `.model(...)` to override preset behavior.

`HistoryPolicy::Bounded` now names `max_good_trials`, `max_bad_trials`, and `recent_bad_trials`.
Custom gamma functions must not request more than `max_good_trials`; a larger request returns
`ParzenError::GammaExceedsHistoryLimit`. Use `HistoryPolicy::Full` when an unbounded custom good set
is required.

## Suggestions and completion

Distribution metadata now lives in `SearchSpace`, so suggestion methods accept only a name and
return `Result`:

```rust
let x = study.suggest_categorical("x")?;
let learning_rate = study.suggest_float("learning_rate")?;
let depth = study.suggest_int("depth")?;
study.complete_trial(score)?;
```

Repeated suggestions for the same name return the pending value instead of resampling. Non-finite
objective values are rejected.

Distribution deserialization now runs the same validation as constructors and rejects unknown
fields. Float domains must have a finite transformed width, and stepped-float grids must have at
most `2^53` exactly addressable positions. Injected stepped floats within four ULPs of a legal grid
point are stored as the canonical grid value.

## Injecting and reading trials

Replace `FrozenTrial` with `TrialInput` for insertion:

```rust
study.add_trial(TrialInput {
    params: vec![("x".into(), ParamValue::Categorical(0))],
    value: baseline,
})?;
```

`Study::trials()` now returns an exact-size iterator of allocation-free `TrialRef` values rather
than a slice. Use `TrialRef::to_record()` for an owned, serializable value.

## Algorithm changes

- `GammaStrategy::Optuna` is `min(ceil(0.1 * n), 25)`.
- `GammaStrategy::Hyperopt` is `min(ceil(0.25 * sqrt(n)), 25)`.
- Acquisition draws candidates from the good model and selects the maximum log likelihood ratio.
- Explicit groups use a shared, trial-aligned mixture component and joint product likelihood.
- Discrete candidates are quantized before scoring and use Gaussian cell probability mass.
- The performance preset has bounded estimator state; completed-trial storage remains complete.
- Equal objectives use seeded trial-ID tie-breaking rather than insertion order.
- v0.1 and v0.2 suggestion sequences are intentionally not identical.
