# Changelog

## [Unreleased]

## [0.3.0] - 2026-08-04

### Changed

- Raise the minimum supported Rust version from 1.88 to 1.89.
- Add optional, default-enabled runtime-dispatched SIMD for continuous TPE acquisition while
  retaining an exact scalar fallback and an implementation-independent public API. The runtime
  policy enables SIMD only for the independently modeled, at-least-four-dimensional continuous
  envelope where measurements cleared the performance gate.
- Prepare estimator metadata once, use a deterministic estimator registry, reuse mixture and model
  workspaces, score candidates positionally, and retain typed full-history parameter columns.
- Reuse bounded-history partition buffers and cache exact discrete likelihood scores, substantially
  reducing allocation churn and repeated probability evaluation.
- Add an unpublished, manual cross-crate comparison and profiling harness. It is retained as review
  evidence in the repository and excluded from the published crate.

### Performance

- On the validated Balthasar continuous envelope, runtime SIMD improved selected workloads by
  approximately 7–13% and reduced retired instructions for fixed continuous suggestion by roughly
  29–30%.
- Reusable storage reduced warmed bounded-cycle allocation from approximately 437 KB per operation
  to 1.55 KB per operation in the measured 4D/1,000-observation case.
- Exact score reuse improved fixed-history integer suggestion by roughly 400× in the measured
  1,000-observation case.

These figures are machine- and scenario-specific. Runtime, memory, and optimization quality were
assessed separately. The public Rust API and tested TPE semantics are preserved, and the fixed
32-seed quality suite found no accepted optimization-quality regression. Equivalent batching and
floating-point ordering may change seeded suggestion sequences between minor versions or target
architectures.

## [0.2.0] - 2026-07-31

- Add validated categorical, float, log-float, integer, stepped-integer, and log-integer distributions.
- Add explicit search spaces, categorical conditions, and grouped multivariate suggestions.
- Replace map-backed trials with packed interned storage and allocation-free history views.
- Add candidate-based expected-improvement acquisition, named gamma strategies, and bounded history.
- Reject invalid configuration, search-space, trial, and non-finite objective inputs with typed errors.
- Add statistical quality, property, public-API, and Criterion performance tests.
- Validate distributions during deserialization and reject non-sampleable finite numeric domains.
- Canonicalize injected stepped floats and use uniform discrete startup priors.
- Score integer and stepped-float kernels by discrete Gaussian probability mass.
- Replace the linear-update rank vector with bounded per-estimator histories or an exact ordered
  full-history index.
- Replace factorized grouped acquisition with a cached trial-aligned mixture of products.

## [0.1.1] - 2026-07-31

- Allow zero startup trials without panicking on an empty study.
- Exclude NaN objective values from TPE fitting and best-trial selection.

## [0.1.0] - 2026-07-31

- Initial categorical Tree-structured Parzen Estimator implementation.
- Deterministic seeded sampler, studies, trials, and maximize/minimize directions.

[Unreleased]: https://github.com/moderately-ai/parzen/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/moderately-ai/parzen/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/moderately-ai/parzen/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/moderately-ai/parzen/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/moderately-ai/parzen/releases/tag/v0.1.0
