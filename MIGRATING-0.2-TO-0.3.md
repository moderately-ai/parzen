# Migrating from Parzen 0.2 to 0.3

Parzen 0.3 preserves the public Rust API and tested TPE behavior of 0.2, so no source-level API
migration is required. Two build and reproducibility changes may require attention.

## Rust version

The minimum supported Rust version is now 1.89. Upgrade the Rust toolchain before resolving Parzen
0.3.

## Default SIMD feature

The new `simd` feature is optional and enabled by default alongside `serde`. It uses Pulp for
runtime-dispatched SIMD without exposing SIMD types in Parzen's public API. Supported machines
select an appropriate implementation at runtime, and all other targets use the scalar fallback.

To retain Serde while using the scalar implementation from Parzen 0.2, configure the dependency as:

```toml
parzen = {
    version = "0.3",
    default-features = false,
    features = ["serde"],
}
```

Use `default-features = false` without an explicit feature list when neither Serde nor SIMD is
needed.

Parzen applies the SIMD acquisition path only to at least four independent continuous estimators,
the envelope where it passed the project's runtime and non-regression gates. One-dimensional,
grouped, categorical, discrete, and mixed estimators retain exact scalar paths. SIMD is an internal
runtime optimization and does not change the configured kernels, candidate count, history policy,
or optimization budget.

## Seeded reproducibility

Parzen remains deterministic for a fixed optimized build, seed, target architecture, search space,
and observation order. Semantically equivalent batching and floating-point evaluation order may
change the exact suggestion sequence between Parzen minor versions or target architectures. Do not
use a cross-version suggestion sequence as a persistent wire or snapshot format.

The complete fixed 32-seed quality suite used for the 0.3 performance work found no accepted
optimization-quality regression. This is a statistical quality result, not a promise of identical
candidate sequences.
