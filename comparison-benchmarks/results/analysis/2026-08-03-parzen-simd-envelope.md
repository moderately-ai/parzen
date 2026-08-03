# Parzen SIMD performance envelope

Date: 2026-08-03

Branch: `perf/parzen-envelope`

Series start: `94d42b4`

Measured production implementation: `c950c7aa47d7563c4b170bd3a6b734670306ccdd`

Evidence/reporting harness: `22c0d71`

Host: Balthasar, AMD Ryzen 7 5800X, Linux x86-64, CPU 7, `schedutil`

Toolchain: Rust 1.89.0

## Decision

Retain the runtime-dispatched Pulp implementation for independent continuous
search spaces with at least four estimators. It is an exact, targeted speedup,
not a blanket SIMD mode:

- Curated 4D/10,000 fixed-history suggestion improved by 12.19% in bounded mode
  and 12.72% in full mode, with all eight paired rounds favoring SIMD.
- Curated 4D/10,000 cycle improved by 7.50% in bounded mode and 9.65% in full
  mode, again with all eight paired rounds favoring SIMD.
- Every measured target point in the history and dimensional envelopes improved.
- All 2,304 paired scalar/SIMD quality payloads were exactly equal, including
  every convergence curve.
- The implementation retired 28.90% fewer instructions and 12.78% fewer cycles
  in the fixed-suggest counter case.

Do not extend this implementation to one-dimensional, grouped, mixed,
categorical, discrete, model-rebuild, or ARM transcendental paths without new
evidence. Attempts in several of those regions were neutral or slower and were
removed before the production commit.

Absolute timings in this report are machine-specific. Scalar and SIMD results
were captured from the same clean commit, on the same pinned CPU, with matching
fixtures and configurations. Runtime, quality, memory, and competitor results
remain separate observations.

## What changed

The public API and TPE semantics are unchanged. The `simd` feature is enabled by
default and selects the pinned `pulp = 0.22.3` dependency. A build without default
features remains the scalar reference.

The acquisition path decides once per estimator whether the search-space shape
is eligible at `src/sampler.rs:519-535`. The policy at
`src/sampler.rs:794-801` requires at least four estimators and requires every one
to be an independent continuous estimator. Candidate generation and winner
selection remain unchanged at `src/sampler.rs:539-641`.

Both good and bad mixtures enter the same batch boundary at
`src/sampler/mixture.rs:448-498`. The selected Pulp kernel is dispatched once at
`src/sampler/vector_math/simd.rs:10-25`, then vectorizes the mixture-component
axis at `src/sampler/vector_math/simd.rs:50-84`. The stable log-sum-exp reduction
uses a vector exponential at `src/sampler/vector_math/simd.rs:87-111`.

The x86-64 exponential is specialized for the exact non-positive log-sum-exp
domain. Its range reduction, degree-13 polynomial, attribution, and ordinary
input range are at `src/sampler/vector_math/simd.rs:116-166`; exceptional lanes
fall back to the platform scalar implementation at
`src/sampler/vector_math/simd.rs:168-179`. Non-x86 platforms retain the scalar
transcendental. Dispatch is runtime-based and needs neither nightly Rust nor
`target-cpu=native`.

Pulp was selected because it supports stable Rust, runtime dispatch, and f64
lanes. The originally evaluated `rten-simd` interface did not expose the required
public f64 operations. A future migration to portable `std::simd` is reasonable
when that API is stable and meets the same numerical and runtime gates.

## Fast-feedback discipline

The harness now exposes `quick`, `checkpoint`, and `curated` protocols, bounded
calibration, case and suite timeouts, planning, deterministic shards, immediate
JSONL flushing, and resume keys. Runs predicted above 45 minutes refuse to start
unless `--allow-long-run` is explicit. No blind sleeps or profiler pauses were
introduced.

The evidence-producing run durations were:

| Work | Protocol | Duration |
|---|---|---:|
| Typical focused scalar/SIMD screen | checkpoint subset | about 50 s |
| Final paired history/dimension/guard matrix | checkpoint | 5 m 46 s |
| Principal 4D/10k and categorical confirmation | curated | 3 m 48 s |
| Complete 32-seed scalar and SIMD quality comparison | sharded quality | about 42 s |
| Four 30-second Samply captures plus guarded setup | profile | 2 m 3 s |
| Five-repeat calibration and counter groups | counters | 14 m 19 s |

Representative direct-binary commands were:

```text
taskset -c 7 comparison-benchmarks/target/release/compare timing \
  --backend parzen --protocol checkpoint --history 10000 \
  --scenario independent-float --output <jsonl>

taskset -c 7 comparison-benchmarks/target/release/compare timing \
  --backend parzen --protocol curated --history 10000 \
  --scenario independent-float --output <jsonl>

taskset -c 7 comparison-benchmarks/target/release/compare quality \
  --backend parzen --output <jsonl>

samply record --save-only --unstable-presymbolicate --rate 4000 \
  -o <profile> -- <profiling-binary> --scenario independent-float \
  --operation profile --history 10000 --dimensions 4 \
  --profile-workload <fixed-suggest|cycle> --profile-seconds 30

comparison-benchmarks/target/release/compare report \
  comparison-benchmarks/results/baselines/2026-08-03-balthasar-simd.jsonl \
  --output comparison-benchmarks/results/baselines/2026-08-03-balthasar-simd.md
```

Scalar and SIMD binaries were built before measurement and invoked directly;
none of the timing or profiling evidence came through `cargo run`.

## Runtime evidence

### Curated acceptance cases

Medians are nanoseconds per logical operation. Improvement is
`(scalar - SIMD) / scalar`.

| Case | Mode | Scalar median | SIMD median | Improvement | Paired wins |
|---|---|---:|---:|---:|---:|
| 4D/10k fixed suggest | bounded | 528,616 | 464,204 | 12.19% | 8/8 |
| 4D/10k fixed suggest | full | 12,430,966 | 10,849,151 | 12.72% | 8/8 |
| 4D/10k cycle | bounded | 782,683 | 724,012 | 7.50% | 8/8 |
| 4D/10k cycle | full | 16,374,342 | 14,794,326 | 9.65% | 8/8 |

### Envelope

The checkpoint screen found the following cycle improvements:

| Shape | Bounded | Full |
|---|---:|---:|
| 4D, history 10 | 13.48% | 14.27% |
| 4D, history 100 | 11.83% | 13.25% |
| 4D, history 1,000 | 8.19% | 8.38% |
| 4D, history 10,000 | 7.50% | 9.65% |
| 4D, history 100,000 | 8.12% | 9.27% |
| 8D, history 1,000 | 6.98% | 8.27% |
| 16D, history 1,000 | 6.07% | 7.23% |

Fixed-suggest improvements were 12.01%/11.84% at 4D/1k,
12.19%/12.72% at 4D/10k, and 12.04%/9.42% at 4D/100k for
bounded/full respectively. The gain is therefore not a single-point anomaly.
The smaller cycle percentage at large dimensions reflects an increasing share
of non-acquisition work, not a reversal in the vector kernel.

### Neutral and guard regions

- One-dimensional continuous estimators intentionally use the scalar policy.
  Their checkpoint differences were noise-level.
- Categorical estimators intentionally use the existing specialized scalar
  marginal. Curated bounded cycle differed by +0.01%; full improved by 2.85%.
  Neither is claimed as a SIMD win.
- Grouped/correlated and mixed estimators use scalar fallback. The worst focused
  non-target observation was a 1.28% correlated full-suggest regression, within
  the 3% guard.
- Integer likelihood remains scalar and exact.

### Competitor context

At independent 4D/1k cycle, medians were:

| Implementation | Median ns/op |
|---|---:|
| Parzen bounded SIMD | 720,889 |
| optimizer | 723,036 |
| tpe | 899,805 |
| Parzen full SIMD | 1,367,311 |
| hyperopt | 1,712,930 |

This is not a claim of universal superiority. Each crate has different kernels,
bandwidth rules, priors, gamma behavior, and wrappers. In particular, Parzen full
history deliberately pays a cost that bounded mode avoids. At 4D/10k cycle,
Parzen bounded was about 0.724 ms, optimizer about 6.990 ms, and Parzen full about
14.794 ms. These results describe the tested lifecycle and semantic configuration
only.

## Samply evidence

All four profiles used Samply 0.13.1, release-derived binaries with full debug
information, 4 kHz sampling, presymbolication, all threads, and preserved symbol
sidecars. The read-only `harness-tools` analyzer was run in default idle-filtered
mode, with `--filter parzen`, and with `--top 40 --min-pct 0.5`.

| Workload | Build | Operations | Observations | Active samples | Idle |
|---|---|---:|---:|---:|---:|
| fixed suggest | scalar | 2,401 | 10,000 -> 10,000 | 120,372 | 0% |
| fixed suggest | SIMD | 2,739 | 10,000 -> 10,000 | 120,384 | 0% |
| cycle | scalar | 1,837 | 10,000 -> 11,837 | 120,337 | 0% |
| cycle | SIMD | 2,052 | 10,000 -> 12,052 | 120,361 | 0% |

All profiles passed symbol, observation-transition, operation-count, idle, and
setup-share checks. Profiled wall time is not used as benchmark evidence.

In fixed-suggest, scalar acquisition occupied 99.73% inclusive and the platform
`exp` leaves accounted for about 85.05%. The SIMD profile moved the work into
`vector_math::simd::continuous_log_pdf_batch` (93.31% inclusive), while platform
`exp` leaves fell to about 41.89%. This agrees with the paired runtime and the
instruction reduction.

The large `fun_79b60` span visible in the browser flame graph is an unresolved
internal glibc/libm symbol beneath `exp`; it is not a Parzen function. Inclusive
attribution is therefore the correct interpretation. The exact Parzen call site
replaced by the vector path is the log-sum-exp reduction at
`src/sampler/vector_math/simd.rs:87-111` (scalar counterpart in
`src/sampler/vector_math/scalar.rs`).

In cycle, scalar acquisition was 72.17% inclusive and model rebuilding 25.74%.
After SIMD, acquisition fell to 63.73% of active samples; rebuilding rose
proportionally to 28.89% because it did not become slower while the acquisition
denominator shrank. The major remaining exact rebuild frame is
`NumericKernels::rebuild` at `src/sampler/mixture.rs:615`, including sorting and
coefficient construction. Exact Gaussian cell/normalization mass is implemented
at `src/sampler/math.rs:7-46` and accounted for 17.76% inclusive in the SIMD cycle
profile. Full-history prepared lookup is at `src/sampler/history.rs:214-281` and
was 0.86% inclusive.

Approximate cost budgets, using the unprofiled 14.794 ms full-cycle median and
profile proportions only as attribution estimates, are:

| Bucket | Approximate upper bound |
|---|---:|
| Acquisition | 9.43 ms/cycle |
| Model rebuild | 4.27 ms/cycle |
| Gaussian mass within rebuild | 2.63 ms/cycle |
| Prepared full-history lookup | 0.13 ms/cycle |
| Remaining wrapper/storage/setup | under 0.31 ms/cycle |

These are upper bounds, not additive optimization promises; inclusive frames
overlap and profiling changes execution cost.

Profile checksums:

| Artifact | SHA-256 |
|---|---|
| scalar fixed profile | `40458a62d82a477bc70778650c7499491700aa1f16afd2ea3cf7aed97d545632` |
| scalar fixed sidecar | `c2781e7d10485f0552dc166418b59c45cd837c26190b1a95750f5fe330c1cb29` |
| SIMD fixed profile | `8192edf9125c0b1b9fb615677bfdf608e84a5f2466940e43c2d39c12f1ee9294` |
| SIMD fixed sidecar | `84b88959eda5b11398b45325c2225da7a25e1c4e40d12521e90437599e35c46c` |
| scalar cycle profile | `1bf8cf6f7b4f082d462bcdf538e325d7ec5addc126c46023138484a25d102180` |
| scalar cycle sidecar | `8020fdd6ee5ab657f5ef1c57a87a465b914e5e6d8a312ed629f7d05b6aacf344` |
| SIMD cycle profile | `84b9c6bdd95b2b431a1793464366530757ca9a0f8664c213acb839d8a2256387` |
| SIMD cycle sidecar | `a0cbcf0434bad51e6870f9d1ce29c33994a3d19649c9ebc0f6c3c41c6f7116c0` |

The profiling binaries were
`e9f5e0cc2f4cd9a188fd3249411744536dc546188b15306ceed638564d8a5321`
(scalar) and
`4ed26748b679140dd2559d88b383fe7c1a01e479779119643989dbcb4d470ec1`
(SIMD).

## Hardware counters and roofline decision

The guarded Balthasar workflow recorded `kernel.perf_event_paranoid=4`, installed
its restoration trap, temporarily set it to 1, and verified restoration to 4.
No governor, service, process, or other sysctl was changed. `dcgm-exporter`
remained active at about 30.6% host CPU and is disclosed contamination.

The FMA calibration measured about 70.79 GFLOP/s, and retired-FP counts agreed
with known work within the 20% gate. The 256 MiB-per-array STREAM-style triad
measured about 27.45 GB/s. However, Balthasar did not support the requested
`nps1_die_to_dram` event. Without validated production DRAM bytes there is no
conventional hardware roofline in this report.

For the fixed 4D/10k suggestion case, median per-operation counters changed:

| Counter | Scalar | SIMD | Change |
|---|---:|---:|---:|
| cycles | 58.04 M | 50.62 M | -12.78% |
| instructions | 101.16 M | 71.92 M | -28.90% |
| branches | 18.33 M | 15.39 M | -16.05% |
| branch misses | 617,926 | 412,155 | -33.30% |
| cache references | 1.381 M | 1.287 M | -6.78% |
| cache misses | 34,652 | 32,343 | -6.66% |
| retired FP operations | 24.43 M | 43.53 M | +78.19% |

IPC fell from about 1.743 to 1.421, but this is not a regression by itself: the
SIMD implementation completes the operation with materially fewer instructions
and cycles while retiring more packed floating-point work. L2 accesses were
stable (+0.65%). The L2-miss group exceeded the 10% coefficient-of-variation gate
(13.55%) and is rejected. All accepted groups had at least 95% running/enabled,
no CPU migrations, nonzero expected events, and at most 10% variation. Counter
file and host-audit checksums are
`7d8b1866845dbea8d790610bf4db4e7fce4fc56baead9659cd1f353a95f99a6b`
and `a634c2dbf4b8055db50ce733a2b2e7650081f34eb26f878f88411d3c91925186`.

Cycle counters also moved in the expected direction: cycles per operation fell
10.62% and instructions fell 20.76%. Because cycle history grows throughout the
ten-second capture, these are supporting mechanism evidence, not primary timing.

## Numerical and quality correctness

The vector exponential has differential tests for ordinary inputs, exceptional
lanes, tails, non-lane-multiple lengths, and scalar-feature builds. The retained
x86 path stayed within four ULP of scalar libm. A deterministic 100,000-input
audit runs normally; the ignored 10,000,000-input audit was run on Balthasar from
the exact evidence branch and passed in 0.16 seconds (11 seconds including test
startup).

The complete quality comparison covered 32 seeds, nine objectives, four budgets,
and both full and bounded modes: 2,304 records per build. Every scalar and SIMD
quality payload matched exactly, so median regret, p90 regret, success rate,
evaluations to thresholds, and every convergence point are unchanged. This is
stronger than the configured quality non-regression gate.

## Memory

SIMD did not change retained data structures, so memory runs characterize the
current algorithm rather than support a memory-improvement claim.

| Case | Mode/backend | Retained after ingest | Cycle bytes/op | Peak live |
|---|---|---:|---:|---:|
| 4D/1k | Parzen full | 197,928 | 121,411 | 873,050 |
| 4D/1k | Parzen bounded | 205,264 | 1,547 | 463,530 |
| 4D/1k | optimizer | 1,247,168 | 147,659 | 1,633,762 |
| integer/1k | Parzen full | 114,832 | 30,891 | 336,240 |
| integer/1k | Parzen bounded | 85,608 | 753 | 175,464 |
| integer/1k | optimizer | 780,922 | 41,109 | 1,120,306 |

Bounded mode remains the flat-memory production choice. Its reusable partition
is at `src/sampler/history.rs:169-190`; it sorts a small bounded scratch set.
Full mode's allocation and rebuild cost continues to grow with history.

## Accepted, rejected, and deferred work

### Accepted

- Fast-feedback protocols, hard timeouts, plan preview, resume, and sharding.
- Prepared estimator/model coordinates and reusable scalar/SIMD workspace from
  the preceding optimization series.
- Stable runtime dispatch through Pulp.
- AVX2/FMA component-axis continuous scoring for eligible independent spaces.
- An exact x86 log-sum-exp exponential with scalar exceptional-lane fallback.
- A policy gate that keeps neutral or adverse shapes on their established paths.
- Schema/report fields that distinguish scalar, policy fallback, and selected ISA.

### Rejected after measurement

- `rten-simd`: required public f64 operations were unavailable.
- Candidate-axis/component-major scoring: 6-11% slower than vectorizing mixture
  components, so it was removed rather than rationalized as infrastructure.
- ARM custom vector exponential: slower than the scalar transcendental; ARM uses
  exact scalar fallback until better evidence or stable `std::simd` support.
- SIMD `erfc`/Gaussian normalization: passed a 100,000-input four-ULP audit but
  made cycle workloads 3-7% slower; completely reverted.
- SIMD discrete likelihood: exact CDF/mass and normalization attempts lost to the
  existing scalar path. The exact duplicate/persistent integer cache at
  `src/sampler.rs:590-625` remains.
- SIMD for categorical, grouped, mixed, and one-dimensional estimators: no
  qualifying win. Policy fallback protects these cases.
- A standalone transcendental microcheck command was not retained. The production
  kernels are deliberately crate-private, and duplicating them in the harness
  would test a copy rather than shipped code. Root differential tests provide the
  mathematical oracle; Balthasar's FMA/bandwidth calibrator remains a hardware
  counter validator, not a Parzen-kernel benchmark.
- Numeric storage alignment changes: no measured memory-layout bottleneck or
  accepted upper bound justified extra representation complexity.
- Parallel acquisition: only 24 candidates are evaluated, and thread launch,
  synchronization, determinism, and single-thread benchmark semantics would make
  this a poor default. It remains out of scope absent contrary evidence.

### Deferred

The next largest measured cost is full-history numeric reconstruction, not more
acquisition SIMD. A future experiment should attack data movement and model
maintenance around `NumericKernels::rebuild`, sorting, and exact Gaussian
normalization. It must preserve exact kernels and pass the existing runtime,
memory, and quality gates. The current 4.27 ms/cycle attribution is an upper
bound; the 3-7% failed normalization experiment shows that a visible profile
share does not guarantee a profitable SIMD implementation.

Incremental full-history continuous maintenance remains worth prototyping only
under the previously defined restrictions (full history, independent continuous
estimators, uniform weights, Optuna gamma, sufficiently large history). The
prototype must beat rebuild after boundary membership changes and neighbor
bandwidth updates are included. Bounded mode should not inherit that complexity;
its cycle time stays essentially flat from 1k through 100k observations.

## Public API decision

No API redesign is justified. Fixed-history profiles place essentially all cost
inside acquisition; cycle profiles place more than 97% in acquisition plus model
rebuild. Study/trial lifecycle, public-value conversion, storage, and validation
remain below both the 10% active-CPU and 25 microsecond thresholds. Batch public
suggestion would also require unresolved pending-trial/fantasization semantics.

## Provenance and artifacts

The tracked baseline contains 5,202 schema-v5 records, all from clean production
commit `c950c7a`:

- Parzen: 5,144 records (paired scalar/SIMD timing and complete quality evidence).
- optimizer: 42 records.
- tpe: 8 records.
- hyperopt: 8 records.

Baseline JSONL SHA-256:
`18dcab8a1f5c1cf0a1636187b3ebd2373f639cbf6674a70e0fbf422d8ea3e7f0`.
Generated Markdown SHA-256:
`3d285b04f6dcb083eb81c645c8aca82f14b0d63ddf9cfbdfc616881775031e2d`.

The scalar and SIMD principal runtime inputs were
`65d7745dbf008383b15e814b9a234903ca0e6607299fe92c3fcf1f52e949b6c6`
and `709103c1c8f37c7ba3a51b41b0ed5a83d043b0a1b02fd38ce8d7c80eec6706d5`;
curated inputs were
`480255dcc2eb98d24d7c9579f44c58ed13515702ceb509421392a10cd687a89e`
and `bd49e96ff7ff418739742f9fb5b526e99a80cd4c1286c4f87832a96f227c823d`;
quality inputs were
`555b1a733a98fbe7354b27f4174cba3eadbb50224b74e68a26a5147e1b76a99e`
and `3c8241cf79dc939e1480959dcdb7dc96ea85aaa0a177179e21ff2b6d265e10c2`.
The focused tpe/hyperopt input is
`31b72a140f7cfc88585d6292f309040c3737d9816ffe9f18d1ee1ce7eac3838d`.
Raw JSONL, Samply profiles and sidecars, DHAT output, counter CSV/JSON, host
audits, and the local HTML walkthrough remain ignored and untracked.

## Final recommendation

Ship this branch for review as a narrowly targeted runtime optimization. The
measured envelope supports Pulp AVX2/FMA acquisition for independent continuous
spaces; it does not support broadening SIMD merely because a loop looks
vectorizable. The next performance phase should focus on full-history model
reconstruction and data movement, with bounded mode preserved as the predictable
latency/memory configuration. Revisit stable `std::simd` when it can replace Pulp
without weakening runtime dispatch, f64 coverage, numerical accuracy, or the
measured gates.
