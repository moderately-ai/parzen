# Parzen runtime optimization investigation

## Integer acquisition before-state

The integer specialization gate is met. These measurements precede production
changes and use commit `7386a50c703a7c9c43af92a36e14e54b6bc4a28f` on Balthasar
(AMD Ryzen 7 5800X, Rust 1.89.0, Samply 0.13.1, `schedutil`, target pinned to
CPU 7). The persistent `dcgm-exporter` load remained present and was not
stopped.

At 1,000 observations, routine release measurements reported:

| Backend | History | Suggest median | Cycle median |
|---|---|---:|---:|
| Parzen | full | 1.317 ms | 1.468 ms |
| Parzen | bounded | 0.683 ms | 0.795 ms |
| optimizer | full | 0.140 ms | not retained in the preliminary run |

The matched 30-second Samply captures were release-derived, used full debug
information, sampled all threads at 4 kHz, and contained approximately 120,000
active samples each with no idle samples. Fixed-suggest observation counts
remained exactly 1,000. Cycle observation growth exactly matched the reported
operation count. `kernel.perf_event_paranoid` started at 4, was temporarily set
to 1, and was verified restored to 4 by an exit trap.

For both full and bounded fixed-suggest, 99.4% or more of active samples were
inclusive in `ProductMixture::log_pdf`; 83.5% were inclusive in
`log_gaussian_mass`. The latter performs the discrete Gaussian cell-mass CDF
calculation. Bounded cycle still attributed 83.1% to that calculation, with
4.8% in `ProductMixture::build` and approximately 4% in sorting. Thus discrete
cell-mass evaluation exceeds both investigation thresholds: 40% of active CPU
and 50 microseconds per suggestion.

This evidence supports deduplicating integer candidates within the acquisition
batch and caching invariant cell/kernel terms. It does not support changing the
Gaussian mass calculation, density semantics, candidate count, or precision.

The raw profiles, symbol sidecars, records, host-change audit, and their SHA-256
checksums remain under the ignored
`comparison-benchmarks/results/raw/profiles/integer-before/` directory.

## Optimization checkpoints

Subsequent sections will record each retained or rejected phase using matched
release timing, Samply, memory, and quality evidence.
