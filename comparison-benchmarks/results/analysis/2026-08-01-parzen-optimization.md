# Parzen runtime optimization investigation

## Outcome

The retained changes improve three distinct performance envelopes without
changing Parzen's public API or any tested suggestion sequence:

- The 4D/1,000 bounded cycle median fell from 849.8 to 770.2 microseconds
  (9.4%). The first split-only checkpoint accounted for about 7.8% and won all
  eight curated before/after rounds.
- Full-history 4D/10,000 cycles fell from 17.46 to 16.41 milliseconds (6.0%).
  Full history remains component-scaled; at 100,000 observations it still takes
  181.95 ms/cycle and should not be presented as a bounded-cost mode.
- Bounded cached integer suggestion fell from 683.2 microseconds to 1.72
  microseconds. Full cached integer suggestion is 1.76 microseconds. The cache
  stores exact good/bad likelihoods for at most 4,096 integer values and is
  cleared on every model generation, so state-growing cycles remain 524.2 and
  651.6 microseconds respectively.

The complete 32-seed gate produced 2,304 records on both the original
`7386a50` build and the optimized build. Every result checksum and complete
quality payload matched exactly. This is stronger than the planned statistical
non-regression gate: the retained implementation did not change the tested
suggestion sequences.

All timings below are machine-specific. Only same-host runs under comparable
load are meaningful. Runtime, allocation, retained memory, and quality remain
separate results.

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

### Bounded partition reuse: retained

`BoundedHistory::split_into` now reuses capacity and the numeric path no longer
materializes retained IDs once merely to split them again. The old profile put
19.37% inclusive CPU in `BoundedHistory::split`; the matched after profile puts
10.51% in `split_into`. Its direct self share fell from 2.69% to 1.52%, while
the release median improved by about 7.8%. See `src/sampler/history.rs:169` and
`src/sampler.rs:378`.

The reusable buffers are sized from bounded-history limits. Generated-history
tests compare every output against the original allocating implementation and
assert stable ordering and capacity ceilings.

### Prepared models and reusable build storage: retained for memory

This phase improved the 4D/1,000 bounded-cycle runtime by only about 2%, below
the standalone timing gate. It passed the memory gate decisively: warmed
bounded cycle allocation fell from approximately 437 KB/op to 1.55 KB/op.
Full mode fell from approximately 703 KB/op to 122 KB/op at 1,000
observations. Product mixtures, numeric kernel arrays, component weights,
transformed values, and sort order now reuse their high-water capacities; see
`src/sampler/mixture.rs:100` and `src/sampler/mixture.rs:419`.

Current 4D/1,000 retained-after-ingest measurements are 200,496 bytes for
bounded Parzen, 191,944 bytes for full Parzen, and 1,247,168 bytes for
`optimizer`. Allocation churn and retained state are intentionally reported as
different quantities.

### Positional batch acquisition: retained with the discrete phase

Positional batching alone improved the 8D correlated numeric case by about
4.5%, just below its standalone gate. It was retained as the required substrate
for discrete reuse, whose combined effect is material. All 24 candidates are
generated into reusable estimator-order storage, eliminating the former
per-parameter linear lookup. Good and bad score arrays are reused, and only the
winning row is converted back to the public parameter/value representation.
See `src/sampler.rs:467` and `src/sampler/mixture.rs:231`.

### Full-history typed columns: retained

Full mode now records validated typed values in parameter-local columns at
completion and performs constant-index reads during model reconstruction. This
removed `TrialStorage::typed_value` from the material full-cycle profile; it
previously owned 5.13% self CPU. At 4D/10,000, the phase improved cycle time by
about 5.4% against its immediate predecessor. See
`src/sampler/history.rs:214` and `src/sampler.rs:419`.

The tradeoff is explicit: full 4D/10,000 retained state after ingestion is
about 2.74 MB. Bounded mode does not allocate these unbounded columns and keeps
using capped retained history.

### Exact discrete likelihood reuse: retained

The before profiles confirmed `log_gaussian_mass` at 83.5% inclusive CPU for
fixed integer suggestion. The implementation now:

- precomputes log inverse sigma during model reconstruction;
- reuses repeated integer scores within a 24-candidate batch;
- retains exact good/bad scores across fixed-history suggestions;
- clears those scores whenever the estimator generation changes; and
- caps the cache at 4,096 distinct integer values.

The relevant paths are `src/sampler.rs:492`, `src/sampler/mixture.rs:487`, and
`src/sampler/mixture.rs:541`. No approximation, reduced precision, CDF change,
or likelihood pruning was introduced.

The eight-round curated fixed-suggest medians are 1.71--1.76 microseconds for
bounded Parzen and 1.76--1.76 microseconds for full Parzen. State-growing cycle
medians remain 523.3--525.3 microseconds bounded and 650.4--653.9 microseconds
full because each completion correctly invalidates the likelihood cache.

## Final runtime envelope

Median release-mode milliseconds per operation at four dimensions:

| History | Full suggest | Bounded suggest | Optimizer suggest | Full cycle | Bounded cycle | Optimizer cycle |
|---:|---:|---:|---:|---:|---:|---:|
| 10 | 0.016 | 0.016 | 0.010 | 0.069 | 0.072 | 0.042 |
| 100 | 0.080 | 0.080 | 0.066 | 0.183 | 0.196 | 0.098 |
| 1,000 | 1.080 | 0.528 | 0.689 | 1.495 | 0.770 | 0.715 |
| 10,000 | 12.264 | 0.503 | 6.942 | 16.409 | 0.763 | 6.894 |
| 100,000 | 181.463 | 0.483 | 135.264 | 181.950 | 0.755 | 135.278 |

At 1,000 observations, cycle scaling is:

| Dimensions | Full | Bounded | Optimizer |
|---:|---:|---:|---:|
| 1 | 0.286 ms | 0.154 ms | 0.145 ms |
| 4 | 1.495 ms | 0.770 ms | 0.715 ms |
| 8 | 3.149 ms | 1.564 ms | 1.698 ms |
| 16 | 6.374 ms | 3.236 ms | 4.422 ms |

The remaining small-history result is not hidden: `optimizer` remains faster
for 1D/1,000 and 4D/1,000 cycles. Bounded Parzen becomes faster as dimension or
history increases and preserves a flat history-size envelope.

Representation-sensitive medians at 1,000 observations:

| Scenario | Full suggest | Bounded suggest | Optimizer suggest | Full cycle | Bounded cycle | Optimizer cycle |
|---|---:|---:|---:|---:|---:|---:|
| Categorical | 0.48 us | 0.48 us | 34.85 us | 32.49 us | 32.41 us | 36.85 us |
| Integer | 1.76 us | 1.72 us | 140.15 us | 651.63 us | 524.16 us | 145.04 us |
| Log float | 180.39 us | 90.39 us | 138.39 us | 289.93 us | 156.83 us | 148.90 us |
| Correlated numeric | 308.29 us | 155.87 us | 568.08 us | 750.18 us | 374.57 us | 606.12 us |

Integer fixed-history strength must not be generalized to integer cycles:
`optimizer` remains much faster for the state-growing lifecycle.

## Optimized Samply evidence

All accepted after profiles used Samply 0.13.1, 4 kHz, every thread, direct
release-derived full-debug binaries, CPU 7 target affinity, and preserved
symbol sidecars. They contained approximately 120,300 active samples and zero
idle samples. Observation transitions matched the selected workload exactly.

At full 4D/10,000 cycle:

- acquisition is 69.1% inclusive;
- `ProductMixture::log_pdf_positional` is 69.0% inclusive;
- model reconstruction is 29.2% inclusive;
- `NumericKernels::rebuild` is 26.9% inclusive; and
- `log_gaussian_mass` is 15.6% inclusive.

The accepted optimized full-cycle profile SHA-256 is
`6bebf6e45828102cc09fce63affa06496c0c716fddcee77335b7073cea941a02`;
its sidecar is
`936dd5dfd320df3c6d6b3b9db373fef45bff3c416a368501cdd87b8711593316`.
The optimized bounded integer-cycle profile is
`80a6dac57a8e970731e5d1994b686b829975d626e69560d2ce747d03473193d6`;
its sidecar is
`ef5484a1fd8095d0377506399da4180a05ce1a819cd2d39964399aaad56af13a`.

## Incremental maintenance decision

The post-optimization full 4D/10,000 profile crosses the investigation trigger:
model rebuilding remains above 20% and far above 100 microseconds, while its
history slope is adverse. Its measured absolute upper bound is approximately
4.8 ms/cycle.

No incremental implementation is retained in this branch. A correct design
must update good/bad membership as gamma changes, preserve grouped trial
alignment, update local bandwidth neighbours, handle the global minimum
bandwidth change below 100 components, renormalize prior/component weights, and
fall back for non-uniform weighting. A partial fast path that ignored any of
those invalidations would silently change the estimator. The remaining 69%
acquisition share also means eliminating reconstruction entirely would not
flatten full-history scaling.

The recommended later prototype is therefore narrowly scoped to independent,
uniform-weight, continuous numeric estimators after 100 observations. It should
live on a separate branch, retain complete rebuild as the oracle and fallback,
and compare every incremental model's means, sigmas, normalization constants,
and likelihoods against that oracle before performance measurement. This is a
separate algorithmic phase, not a local optimization suitable for inclusion
without those invariants.

## Provenance and artifacts

- Branch: `perf/parzen-envelope`
- Original comparison commit: `7386a50c703a7c9c43af92a36e14e54b6bc4a28f`
- Optimized production commit: `6e10121`
- Final focused-harness commit used for the envelope: `4bfa0e6`
- Host: Balthasar, AMD Ryzen 7 5800X, Linux 6.8.0-110-generic
- Toolchain: Rust 1.89.0
- Governor: `schedutil`; target affinity: CPU 7
- Persistent contamination: `dcgm-exporter` at approximately 30.7% of one CPU

`kernel.perf_event_paranoid` started at 4 for every profiling group. Guarded
exit traps restored the exact original value and verified a final value of 4.
No process, service, governor, or other sysctl was changed.

The curated baseline contains 2,546 JSONL records. Its SHA-256 is
`267138f4311e4d8b7e6b02aa49455f0b99549f7663efbc141689b4cd6f51813e`.
The generated Markdown SHA-256 is
`4b255bd8dd6bcfd306eba57561149e16ae62dfadcd02b82f754a23ab188543b9`.
Raw JSONL, DHAT files, profiles, sidecars, worktree comparisons, and host logs
remain ignored evidence.
