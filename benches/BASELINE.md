# Performance baseline

Measured in release mode on an Apple M4 Max (`aarch64-apple-darwin`, Darwin 27.0.0) with Rust
1.88.0 and LLVM 20.1.5 on 2026-07-31. Default crate features were enabled. Treat absolute times as
host-specific; use the checked-in Criterion suite for comparisons on other machines.

## Parzen 0.1.1

The v0.1 path rebuilt and rescanned the full model on every suggestion.

| Trials | Choices | Suggestion |
|---:|---:|---:|
| 10 | 5 | 0.25 µs |
| 100 | 20 | 6.39 µs |
| 1,000 | 20 | 63.46 µs |
| 10,000 | 20 | 938.84 µs |
| 10,000 | 100 | 4,304.24 µs |

## Parzen 0.2.0

Criterion warmed-model medians:

| Trials | Distribution | Suggestion |
|---:|---|---:|
| 10,000 | categorical, 20 choices | 0.53 µs |
| 10,000 | categorical, 100 choices | 0.68 µs |
| 10,000 | bounded linear float, 24 EI candidates | 34.2 µs |
| 10,000 | grouped linear float, 2 dimensions | 43.3 µs |
| 10,000 | grouped linear float, 8 dimensions | 63.9 µs |

An extended calibration used Criterion 0.8.2 with a 0.5-second warm-up, 1-second measurement, and
10 samples to establish the additional benchmark cases:

| Existing trials | Case | Median |
|---:|---|---:|
| 0 | cold first continuous suggestion | 0.41 µs |
| 10,000 | grouped categorical, 2 dimensions | 46.1 µs |
| 10,000 | grouped categorical, 8 dimensions | 81.6 µs |
| 10,000 | mixed categorical/continuous/integer/stepped group | 927.6 µs |
| 10,000 | conditional integer suggestion | 503.6 µs |
| 10,000 | exact full-history suggest plus completion | 1,392.9 µs |

Before the bounded-history remediation, a separate 10,000-trial, 100-choice probe measured:

- first cold model build: 285 µs
- warmed suggestion: 1.98 µs
- suggest plus completion after each new observation: 17.3 µs

The corrected bounded mixture implementation measured suggest plus completion at:

- 10,000 existing trials: 40.5 µs
- 100,000 existing trials: 36.8 µs
- 1,000,000 existing trials: 32.2 µs

The absence of trial-count growth reflects bounded per-estimator retention. Exact full-history models
remain linear by design.

Packed raw history for 100,000 trials with four categorical parameters reserves 9,437,184 bytes.
This excludes the immutable search space, sampler caches, and allocator metadata, as documented by
`Study::history_capacity_bytes`. `Study::estimator_history_len` reports retained sampler references
separately; under the performance preset each independent estimator retains at most 537 references.

Run:

```bash
cargo bench --bench suggestion
cargo test --test quality --release
```
