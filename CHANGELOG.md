# Changelog

## [Unreleased]

- Raise the minimum supported Rust version from 1.88 to 1.89.

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

[Unreleased]: https://github.com/moderately-ai/parzen/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/moderately-ai/parzen/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/moderately-ai/parzen/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/moderately-ai/parzen/releases/tag/v0.1.0
