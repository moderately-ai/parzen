# Parzen SIMD performance envelope

Date: 2026-08-03

Branch: `perf/parzen-envelope`

Series start: `94d42b4`

Measured and reported commit: `302605766335c9d3cd78e10e906376738f498037`

Host: Balthasar, AMD Ryzen 7 5800X, Linux x86-64, CPU 7, `schedutil`

Toolchain: Rust 1.89.0

## Decision

Retain the runtime-dispatched Pulp implementation for independent continuous
search spaces with at least four estimators. It is an exact, targeted speedup,
not a blanket SIMD mode:

- Curated 4D/10,000 fixed-history suggestion improved by 12.80% in bounded mode
  and 11.48% in full mode; eight of eight and seven of eight paired rounds,
  respectively, favored SIMD.
- Curated 4D/10,000 cycle improved by 7.32% in bounded mode and 9.18% in full
  mode, with all eight paired rounds favoring SIMD.
- Every measured target point in the history and dimensional envelopes improved.
- All 2,560 paired scalar/SIMD quality payloads were exactly equal, including
  every convergence curve.
- On the exact reported commit, the implementation retired 29.0% fewer
  instructions and 11.4% fewer cycles in the full fixed-suggest case. Bounded
  fixed-suggest retired 30.3% fewer instructions and 10.7% fewer cycles.

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

The x86-64 exponential is specialized for the non-positive log-sum-exp
domain. Its range reduction, degree-13 polynomial, attribution, and ordinary
input range are at `src/sampler/vector_math/simd.rs:116-166`; exceptional lanes
fall back to the platform scalar implementation at
`src/sampler/vector_math/simd.rs:168-179`. Non-x86 platforms retain the scalar
transcendental. Dispatch is runtime-based and needs neither nightly Rust nor
`target-cpu=native`.

Pulp was selected because it supports stable Rust, runtime dispatch, and f64
lanes. The originally evaluated `rten-simd` interface did not expose the required
public f64 operations. The committed dependency enables Pulp's `x86-v3` target;
AVX-512 is deliberately not enabled without a validating host. A future migration
to portable `std::simd` is reasonable when that API is stable and meets the same
numerical and runtime gates.

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
| Complete 32-seed scalar and SIMD quality comparison | four shards/build | about 42 s |
| Four full-history 30-second Samply captures | profile | 2 m 3 s |
| Four bounded-history 30-second Samply captures | profile | 2 m 3 s |
| Exact-commit full core counters, 20 x 10 s | counters | 3 m 22 s |
| Exact-commit bounded core counters, 20 x 10 s | counters | 3 m 22 s |

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

The final timing command above was expanded deterministically over these exact
matrices; each child was pinned with `taskset -c 7`, flushed immediately, and is
identified by its JSON record:

| Shard | Protocol | Matrix | Elapsed |
|---|---|---|---:|
| principal | curated | independent float, 4D/10k, suggest + cycle, full + bounded, scalar + SIMD | 3 m 48 s |
| history | checkpoint | 4D at 10, 100, 1k, 100k; suggest + cycle; scalar + SIMD | under 2 m |
| dimension | checkpoint | 1D, 8D, 16D at 1k; suggest + cycle; scalar + SIMD | under 2 m |
| guards | checkpoint | categorical, conditional, correlated 4D, integer, log, mixed, stepped integer/float; suggest + cycle; scalar + SIMD | under 2 m |
| competitors | checkpoint/curated | all four at 4D/1k; optimizer at 4D/10k; integer/1k | under 2 m |
| focused | quick/checkpoint | integer cardinalities 8..100,001; correlated 8D; stepped float | under 1 m |
| quality | four shards/build | seeds 1-8, 9-16, 17-24, 25-32; all configured scenarios/budgets/modes | about 42 s |
| memory | manual DHAT build | independent 4D/1k and integer/1k; full, bounded, optimizer | under 1 m |

The two exact counter commands were `perf stat -x, -e cycles,instructions --
taskset -c 7 <profiling-binary> ... --profile-seconds 10`, repeated five times
for each scalar/SIMD and fixed-suggest/cycle combination, separately for full and
bounded history. The complete expanded arguments and durations remain in the
ignored run logs and JSON configuration fields. No shard timed out and no
`--allow-long-run` override was needed.

## Runtime evidence

### Curated acceptance cases

Medians are nanoseconds per logical operation. Improvement is
`(scalar - SIMD) / scalar`.

| Case | Mode | Scalar median | SIMD median | Improvement | Paired wins |
|---|---|---:|---:|---:|---:|
| 4D/10k fixed suggest | bounded | 528,339 | 460,690 | 12.80% | 8/8 |
| 4D/10k fixed suggest | full | 12,429,088 | 11,002,496 | 11.48% | 7/8 |
| 4D/10k cycle | bounded | 782,518 | 725,226 | 7.32% | 8/8 |
| 4D/10k cycle | full | 16,381,463 | 14,877,496 | 9.18% | 8/8 |

### Envelope

The checkpoint screen found the following cycle improvements:

| Shape | Bounded | Full |
|---|---:|---:|
| 4D, history 10 | 11.01% | 12.69% |
| 4D, history 100 | 11.57% | 13.24% |
| 4D, history 1,000 | 7.50% | 8.68% |
| 4D, history 10,000 | 7.32% | 9.18% |
| 4D, history 100,000 | 7.25% | 8.51% |
| 8D, history 1,000 | 6.75% | 8.57% |
| 16D, history 1,000 | 5.76% | 7.47% |

Fixed-suggest improvements were 11.77%/11.23% at 4D/1k,
12.80%/11.48% at 4D/10k, and 12.40%/8.46% at 4D/100k for
bounded/full respectively. The gain is therefore not a single-point anomaly.
The smaller cycle percentage at large dimensions reflects an increasing share
of non-acquisition work, not a reversal in the vector kernel.

### Neutral and guard regions

- One-dimensional continuous estimators intentionally use the scalar policy.
  Their checkpoint differences were noise-level.
- Categorical estimators intentionally use the existing specialized scalar
  marginal. Checkpoint differences were between 1.40% and 3.37%; they are build
  noise rather than a SIMD claim because both binaries selected scalar fallback.
- Grouped/correlated and mixed estimators use scalar fallback. The worst focused
  non-target observation was a 2.37% log-float bounded-suggest regression, within
  the 3% guard. Correlated 4D ranged from 0.09% to 1.48% slower.
- Integer likelihood remains scalar and exact.

### Competitor context

At independent 4D/1k cycle, medians were:

| Implementation | Median ns/op |
|---|---:|
| optimizer | 711,541 |
| Parzen bounded SIMD | 739,504 |
| tpe | 877,219 |
| Parzen full SIMD | 1,393,453 |
| hyperopt | 1,717,999 |

This is not a claim of universal superiority. Each crate has different kernels,
bandwidth rules, priors, gamma behavior, and wrappers. In particular, Parzen full
history deliberately pays a cost that bounded mode avoids. At 4D/10k cycle,
Parzen bounded was about 0.725 ms, optimizer about 7.025 ms, and Parzen full about
14.877 ms. These results describe the tested lifecycle and semantic configuration
only.

The adverse regions are equally important. In the exact integer-domain sweep,
Parzen bounded cycle rose from about 0.143 ms at cardinality 8 to 0.708 ms at
100,001, while `optimizer` stayed around 0.143-0.147 ms. Parzen full reached
1.375 ms. For stepped floats at cardinality 41, bounded/full cycle measured
0.688/1.271 ms versus `optimizer` at 0.145 ms. Fixed-history Parzen integer
suggestion is exceptionally cheap for small domains because exact scores are
cached, but that does not erase the state-growing rebuild gap. These losses are
why the rejected discrete-SIMD experiments are documented rather than hidden.

For an equivalent correlated numeric 8D case, bounded Parzen measured 0.200 ms
for fixed suggestion and 0.642 ms for cycle, compared with `optimizer` at
0.921 ms and 0.949 ms. Full Parzen was 0.395 ms and 1.179 ms. This grouped path
uses scalar fallback; the result reflects the prepared/reused model work from the
earlier series, not the new continuous SIMD kernel.

## Samply evidence

All eight accepted profiles used Samply 0.13.1, release-derived binaries with full debug
information, 4 kHz sampling, presymbolication, all threads, and preserved symbol
sidecars. The read-only `harness-tools` analyzer was run in default idle-filtered
mode, with `--filter parzen`, and with `--top 40 --min-pct 0.5`.

| History | Workload | Build | Operations | Observations | Active samples | Idle |
|---|---|---|---:|---:|---:|---:|
| full | fixed suggest | scalar | 2,363 | 10,000 -> 10,000 | 120,446 | 0% |
| full | fixed suggest | SIMD | 2,690 | 10,000 -> 10,000 | 120,405 | 0% |
| full | cycle | scalar | 1,830 | 10,000 -> 11,830 | 120,267 | 0% |
| full | cycle | SIMD | 2,006 | 10,000 -> 12,006 | 120,362 | 0% |
| bounded | fixed suggest | scalar | 56,089 | 10,000 -> 10,000 | about 120,300 | 0% |
| bounded | fixed suggest | SIMD | 62,923 | 10,000 -> 10,000 | about 120,300 | 0% |
| bounded | cycle | scalar | 51,389 | 10,000 -> 61,389 | about 120,300 | 0% |
| bounded | cycle | SIMD | 57,973 | 10,000 -> 67,973 | about 120,300 | 0% |

All profiles passed symbol, observation-transition, operation-count, idle, and
setup-share checks. Profiled wall time is not used as benchmark evidence.

In full fixed-suggest, scalar acquisition occupied 99.68% inclusive and the
platform `exp` attribution accounted for 84.12%. The SIMD profile moved work
into `vector_math::simd::continuous_log_pdf_batch` (92.10% inclusive), while
platform `exp` attribution fell to 41.12%. Bounded fixed-suggest shows the same
shape: acquisition was 99.64% inclusive before SIMD, and the scalar batch's
98.83% inclusive attribution is replaced by the Pulp kernel. This agrees with the paired runtime and the
instruction reduction.

The large `fun_79b60` span visible in the browser flame graph is an unresolved
internal glibc/libm symbol beneath `exp`; it is not a Parzen function. Inclusive
attribution is therefore the correct interpretation. The exact Parzen call site
replaced by the vector path is the log-sum-exp reduction at
`src/sampler/vector_math/simd.rs:87-111` (scalar counterpart in
`src/sampler/vector_math/scalar.rs`).

In full cycle, scalar acquisition was 72.35% inclusive and model rebuilding
25.67%. After SIMD, acquisition fell to 63.30% of active samples; rebuilding rose
proportionally to 28.11% because it did not become slower while the acquisition
denominator shrank. The major remaining exact rebuild frame is
`NumericKernels::rebuild` at `src/sampler/mixture.rs:615`, including sorting and
coefficient construction. Exact Gaussian cell/normalization mass is implemented
at `src/sampler/math.rs:7-46` and accounted for 17.35% inclusive in the SIMD cycle
profile. Full-history prepared lookup is at `src/sampler/history.rs:214-281` and
was under 1% self time.

Bounded cycles expose a different remaining envelope: after SIMD, acquisition is
48.31% inclusive, model build 35.64%, numeric rebuild 27.85%, sorting 15.80%,
bounded splitting 11.11%, and prepared value lookup 3.60% self. Those percentages
rose because acquisition became cheaper; they identify the next work, not a
regression caused by SIMD.

Approximate cost budgets, using the unprofiled 14.877 ms full-cycle median and
profile proportions only as attribution estimates, are:

| Bucket | Approximate upper bound |
|---|---:|
| Acquisition | 9.42 ms/cycle |
| Model rebuild | 4.18 ms/cycle |
| Gaussian mass within rebuild | 2.58 ms/cycle |
| Prepared full-history lookup | 0.13 ms/cycle |
| Remaining wrapper/storage/setup | under 0.31 ms/cycle |

These are upper bounds, not additive optimization promises; inclusive frames
overlap and profiling changes execution cost.

An initial bounded capture incorrectly pinned the Samply recorder itself with
`taskset`. It produced zero active samples and empty sidecars, was rejected, and
is not in the baseline. The accepted command leaves Samply unrestricted and pins
only the target binary. This failure and correction are retained in ignored raw
evidence rather than silently discarded.

Accepted full-history profile checksums:

| Artifact | SHA-256 |
|---|---|
| scalar fixed profile | `a6bbdf36621b3c9414aa11a566b94ec2d132ce960328bb1331fd093aa1880665` |
| scalar fixed sidecar | `a478e017a230e04f6df408171c12cc0d4b8955c38857275cdce14d9206bc23ff` |
| SIMD fixed profile | `46f3a7a100cd5d920781bd640baccac0bc1294e1e61a582226ef80323eb31580` |
| SIMD fixed sidecar | `857a2558175044f63ce2c1feaca4ba838feaf2ed40eb537f0261155f245e06d7` |
| scalar cycle profile | `7369e8940f8fc6ee676879d5edc23c0bbad266238d12eb70226f0da0582217a7` |
| scalar cycle sidecar | `43c666ee28a2668fc82320887ca3ce45840cb79fe96d88bc33a506cc17653aeb` |
| SIMD cycle profile | `0e8bba848439fc9cb87a41fb244c5fea511c8f5e58092b1512ddc1ea7ad6aecc` |
| SIMD cycle sidecar | `568ff1b29ecfded3cfa9a29a445b30167b5a9873f00c5a3a94f9689294199556` |

The exact-commit profiling binaries were
`27fa08e0ce05335873db993f33d43e4b2c5ff9c44afe36d8badd8f76c91d99bb`
(scalar) and
`9eabed3de3500d9386c01104feb14e6fb2a9142780b668b1c7cb7f22d08e8d32`
(SIMD). Bounded profile and sidecar checksums are preserved in the raw evidence
manifest and were independently verified before analysis.

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

The broad calibration established the roofline decision. The final exact-commit
confirmation then repeated the non-multiplexed cycles/instructions group five
times for full and bounded fixed-suggest and cycle. Every accepted event had
100% running/enabled and coefficient of variation below 1.9%.

For full-history 4D/10k, mean per-operation counters changed:

| Counter | Scalar | SIMD | Change |
|---|---:|---:|---:|
| fixed-suggest cycles | 58.010 M | 51.385 M | -11.42% |
| fixed-suggest instructions | 100.800 M | 71.616 M | -28.95% |
| cycle cycles | 76.994 M | 69.633 M | -9.56% |
| cycle instructions | 148.183 M | 117.347 M | -20.81% |

For bounded history, fixed-suggest cycles fell 10.7% and instructions 30.3%; cycle
cycles fell 11.2% and instructions 25.4%. IPC falls because the vector kernel
retires much less scalar work; completion time and total cycles both improve.
Earlier accepted event groups also showed fewer branches, branch misses, and
cache events, plus more retired packed floating-point work. The noisy L2-miss
group exceeded its 10% variation gate and remains rejected.

Because cycle history grows throughout each ten-second capture, cycle counters
are supporting mechanism evidence, not primary timing. Exact-commit full counter
manifest/audit checksums are `489bb4f0a871fe3d41b12238099fbd3e22e0e6ff82d5ec4770a62b022713182f`
and `2b547361304782c7a34ac7e0e3894d19818ab92a9db5b91466f54ae83b189336`;
bounded values are `8e2d34cdb306f84d6c9d04985a1326da10c3797086d4e5dad4fb0341d75020be`
and `16330eccfc57c9d692f0c8fb4b4adda1bd75fe85cf4a1129e45b6cf7e05a4309`.

## Numerical and quality correctness

The vector exponential has differential tests for ordinary inputs, exceptional
lanes, tails, non-lane-multiple lengths, and scalar-feature builds. The retained
x86 path stayed within four ULP of scalar libm. A deterministic 100,000-input
audit runs normally; the ignored 10,000,000-input audit was run on Balthasar from
the exact reported commit and passed in 2.61 seconds (about 17 seconds including
a fresh test build).

The complete quality comparison covered 32 seeds, ten objectives, four budgets,
and both full and bounded modes: 2,560 records per build, split into four actual
640-record shards per build. Every scalar and SIMD
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

### Portability boundary

Scalar and SIMD-feature builds both compile for `wasm32-unknown-unknown`; the SIMD
build was also checked with `-C target-feature=+simd128`. These are compile gates,
not browser performance claims. Clean AArch64 builds selected `pulp-neon`, while
ordinary transcendentals remain scalar by policy. An early Apple M4 screen looked
promising, but the attempted curated orchestration overlapped another run and a
background compiler; that data was rejected. No curated NEON claim is made.
AVX-512 remains disabled because this series had no suitable host for its
numerical and runtime acceptance gates.

### Accepted

- Fast-feedback protocols, hard timeouts, plan preview, resume, and sharding.
- Protocol-specific bounded calibration and cross-round calibration reuse, with
  the chosen iteration count and reuse status recorded in schema version 6.
- Prepared estimator/model coordinates and reusable scalar/SIMD workspace from
  the preceding optimization series.
- Stable runtime dispatch through Pulp.
- AVX2/FMA component-axis continuous scoring for eligible independent spaces.
- A four-ULP-contracted x86 log-sum-exp exponential with scalar
  exceptional-lane fallback.
- A policy gate that keeps neutral or adverse shapes on their established paths.
- Schema/report fields that distinguish scalar, policy fallback, and selected ISA.

### Rejected after measurement

- `rten-simd`: required public f64 operations were unavailable.
- Candidate-axis/component-major scoring: 6-11% slower than vectorizing mixture
  components, so it was removed rather than rationalized as infrastructure.
- ARM custom vector exponential: slower than the scalar transcendental; ARM uses
  exact scalar fallback until better evidence or stable `std::simd` support.
- The contaminated Apple M4 curated run: overlapping benchmark/build activity
  invalidated it, so it supports no retained performance claim.
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
memory, and quality gates. The current 4.18 ms/cycle attribution is an upper
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
inside acquisition. In cycles, the named acquisition, rebuild, sorting,
partition, and exact-math frames explain the material envelope; no public
Study/trial lifecycle frame reaches the analyzer's 0.5% reporting threshold.
That is decisively below the required 10% active-CPU gate, so the conjunctive
10%/25-microsecond redesign threshold is not met even though a 25-microsecond
bound cannot be resolved at every full-history point from samples alone. Batch
public suggestion would also require unresolved pending-trial/fantasization
semantics.

## Provenance and artifacts

The tracked baseline contains 5,878 schema-v6 records, all from clean commit
`3026057`:

- Parzen: 5,788 records (paired scalar/SIMD timing, complete quality evidence,
  and eight accepted profile records).
- optimizer: 74 records.
- tpe: 8 records.
- hyperopt: 8 records.

Baseline JSONL SHA-256:
`05f3fe7262bc58daee9df2475fee944db76cff2523ddb079a29fa87cd594794a`.
Generated Markdown SHA-256:
`126e25fe635422b5506f48efbdfa3ab7c37c52c632683bf3a6630d32c08d9c8a`.
Every record reports schema 6, the exact commit above, and `git_dirty=false`;
there are no execution-error records. The baseline additionally covers the
cardinality sweep, correlated 8D, stepped float, DHAT memory cases, and accepted
full/bounded profile metadata. Raw inputs can be reconstructed from each record's
binary checksum, result checksum, protocol, shard, fixture checksum, and complete
configuration.

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
