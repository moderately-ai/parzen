# Parzen performance-envelope investigation

## Conclusion

Parzen has three materially different runtime envelopes. They should not be
optimized as though they were one problem.

1. **Bounded history is already the right large-history architecture.** At four
   dimensions and 100,000 completed trials it held cached suggestion to 0.482 ms
   and cycle time to 0.832 ms. Full Parzen took 189 ms/cycle and `optimizer`
   took 135 ms/cycle. This is the clearest result in the investigation.
2. **Bounded cycle overhead at 1,000 trials is real and actionable.** It was
   19.2% slower than `optimizer` in the curated run, losing all eight rounds.
   Samply attributes about 19% of bounded-cycle CPU to `BoundedHistory::split`,
   including a redundant materialization and repeated sorting. That bucket is
   approximately 163 microseconds against a 135 microsecond gap.
3. **Full-history cost is fundamentally component-scaled.** Exact mixture
   scoring and model reconstruction both grow with retained observations.
   At 10,000 trials, approximately 65% of full-cycle CPU was acquisition and
   31% was model construction. Improving only allocations or trial lookup will
   not flatten this curve.
4. **Cached numeric acquisition is dominated by exponential math.** Across
   4D/1,000, 4D/10,000, and 16D/1,000 captures, 98--100% of fixed-suggest CPU
   was inside `ProductMixture::log_pdf`, with 84--88% reaching exponential
   functions through `logsumexp`. The most promising redesign is batched,
   contiguous candidate/component scoring that preserves exact density
   semantics and makes vectorization possible.
5. **Allocation churn is contributory, not the sole root cause.** DHAT measured
   703 KB/cycle for full Parzen and 437 KB/cycle for bounded Parzen versus
   148 KB/cycle for `optimizer`, but allocator self time was only about 2% in
   cycle profiles. Reusing buffers will help model construction and memory
   traffic, but cannot remove the dominant exponential work.

The recommended implementation order is: remove duplicate bounded splitting,
reuse bounded split/build buffers, introduce estimator-local typed observation
columns, then prototype batch-oriented exact acquisition. Incremental
full-history KDE maintenance should be considered only after those changes are
profiled. No public API change is justified by the present evidence.

## Provenance and limits

- Branch: `bench/comparison-harness`
- Profiled commit: `f20a4b770a8a95043e8a4504011be05469b4ff0f`
- Host: Balthasar, AMD Ryzen 7 5800X, Linux 6.8.0-110-generic, x86-64
- Toolchain: `rustc 1.89.0 (29483883e 2025-08-04)`
- Profiler: Samply 0.13.1, 4,000 Hz, all threads
- Build: release-derived `profiling` profile with full debug information and no
  stripping
- Target affinity: CPU 7 for benchmark and profiled binaries
- CPU governor: `schedutil`
- Background load: `dcgm-exporter` consistently used about 30.7% of one logical
  CPU; it was disclosed and not stopped
- Local date: 2026-08-01; UTC captures crossed into 2026-08-02

Absolute timings are machine-specific. Only same-host runs under comparable
load and build settings should be compared. Profiled wall time and profile
operation counts are validation metadata, not timing evidence.

The JSONL baseline contains 1,177 records. Schema-3 curated timing, quality, and
memory records were migrated mechanically to schema 4 by adding
`config.profile_workload = "cycle"` and null top-level profile fields. Their
measurements, environment data, fixture checksums, and result checksums were not
changed. The baseline JSONL SHA-256 is
`80f0a151435162e73103c66689e19506f094e03642fe2500191f28027853bc5b`.

## Host changes and rejected evidence

`kernel.perf_event_paranoid` began at `4`. Each profiling script installed an
exit trap before setting it to `1`, restored the recorded original value, and
verified a final value of `4`. No governor, process, service, or other sysctl was
changed.

The first six captures were rejected. Samply itself had been pinned to CPU 7,
leaving its collector unable to run; each output contained zero samples and an
empty 38-byte sidecar. A two-second discriminating probe showed that Linux
`perf stat` collected 9,717,121,208 cycles from the target while Samply still
collected no samples. Leaving Samply unpinned and applying `taskset -c 7` only
to the target produced a nonempty 93 KB profile and 54 KB symbol sidecar. All
accepted captures use that arrangement. The rejected hashes and both sysctl
restorations remain in the ignored raw host logs.

One Hyperopt integer `suggest` invocation exceeded the 120-second child timeout
and was retained as a structured timeout. A subsequent integer-cycle invocation
was canceled rather than repeat the same wait. Its partial file was excluded
from the baseline.

## Commands

Release measurements used compiled binaries and the routine protocol:

```text
taskset -c 7 comparison-benchmarks/target/release/compare scaling \
  --scenario independent-float --backend all \
  --rounds 3 --samples 5 --warmup 1 --calibration-ms 100 \
  --timeout-seconds 120 --machine-label balthasar-5800x-cpu7 ...

taskset -c 7 comparison-benchmarks/target/release/compare timing \
  --scenario independent-float --operation cycle --history <N> \
  --backend all --rounds 3 --samples 5 --warmup 1 \
  --calibration-ms 100 --timeout-seconds 120 ...
```

The 4D/1,000 adverse case used the curated protocol of three warmups, 250 ms
calibration, ten samples, and eight rotated rounds. Missing 1D, 8D, and 16D
cycle points were run through the three compiled target binaries in manually
rotated serial order with the same routine parameters; round and invocation
order were added during JSONL aggregation.

Accepted profiles used:

```text
cargo build --manifest-path comparison-benchmarks/Cargo.toml \
  --profile profiling --bins

samply record --save-only --unstable-presymbolicate --rate 4000 \
  -o <name>.profile.json.gz -- \
  taskset -c 7 comparison-benchmarks/target/profiling/<binary> \
  --scenario independent-float --operation profile \
  --profile-workload <fixed-suggest|cycle> \
  --history <N> --dimensions <D> --profile-seconds 30 \
  --parzen-history <full|bounded> \
  --machine-label balthasar-5800x-cpu7 --format json
```

Every profile was analyzed with the read-only `harness-tools` script:

```text
python3 ~/workspace/github.com/tomsanbear/harness-tools/skills/\
samply-profiling/scripts/analyze_profile.py \
  <profile> --top 40 --min-pct 0.5
```

All accepted profiles contained approximately 120,300 active samples and zero
or one idle sample. Fixed-suggest records began and ended at the same completed
observation count. Cycle records grew by exactly their reported operation
count. Every record reported CPU affinity `7`.

## Runtime envelope

### History scaling at four dimensions

Minimum release-mode milliseconds per operation:

| History | Full suggest | Bounded suggest | Optimizer suggest | Full cycle | Bounded cycle | Optimizer cycle |
|---:|---:|---:|---:|---:|---:|---:|
| 10 | 0.020 | 0.020 | 0.010 | 0.078 | 0.085 | 0.041 |
| 100 | 0.083 | 0.083 | 0.066 | 0.198 | 0.222 | 0.097 |
| 1,000 | 1.086 | 0.530 | 0.681 | 1.547 | 0.842 | 0.706 |
| 10,000 | 12.356 | 0.503 | 6.895 | 17.355 | 0.835 | 6.864 |
| 100,000 | 189.061 | 0.482 | 135.072 | 189.739 | 0.832 | 134.878 |

The 1,000-cycle row uses the stricter curated run. `optimizer` won all eight
rounds. Bounded Parzen was 19.2% slower; full Parzen was 119.0% slower.

Bounded Parzen becomes the fastest selected implementation between 1,000 and
10,000 observations and then remains flat. This is a real algorithm/data
structure advantage, not a reason to obscure its small-history overhead.

### Dimensional scaling at 1,000 observations

Minimum release-mode milliseconds per operation:

| Dimensions | Full suggest | Bounded suggest | Optimizer suggest | Full cycle | Bounded cycle | Optimizer cycle |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 0.169 | 0.086 | 0.133 | 0.295 | 0.167 | 0.145 |
| 4 | 1.086 | 0.530 | 0.681 | 1.547 | 0.842 | 0.706 |
| 8 | 2.269 | 1.144 | 1.632 | 3.283 | 1.704 | 1.679 |
| 16 | 4.675 | 2.333 | 4.350 | 6.750 | 3.528 | 4.383 |

Bounded Parzen's cycle crosses from 15.8% slower at one dimension to 19.5%
faster at sixteen. Optimizer's profile explains that crossover: at sixteen
dimensions, distribution matching and hash-map iteration consumed about 63% of
its active CPU.

These are independent estimators, so each Parzen candidate slice has one value.
The `candidate.iter().find(...)` lookup is therefore not dimension-squared in
this envelope. Its grouped-model cost was not isolated and is not a current
optimization priority.

### Representation-sensitive cases

- Categorical cached suggestion: Parzen 0.633--0.635 microseconds,
  `tpe` 3.886 microseconds, and `optimizer` 33.711 microseconds. Parzen's
  categorical marginal is effective. Cycle completion/rebuild raises Parzen to
  51--54 microseconds, while `tpe` takes 7.5 microseconds.
- Log float: bounded Parzen cached suggestion was 0.184 ms versus optimizer at
  0.272 ms; bounded cycle was 0.323 ms versus optimizer at 0.290 ms.
- Correlated numeric: bounded grouped Parzen took 0.317 ms/suggest and
  0.791 ms/cycle versus optimizer multivariate TPE at 1.120 and 1.188 ms.
- Integer cached suggestion: bounded Parzen took 0.681 ms versus optimizer at
  0.140 ms. This is a genuine adverse result, but no integer Samply capture was
  taken, so its precise root cause remains an explicit follow-up question.

## CPU cost budgets

The following budgets multiply unprofiled minimum ns/op by inclusive Samply
shares. They are approximate: profiling changes wall time, and cycle captures
grow history for 30 seconds. The selected high-level scopes are siblings and
account for 94--99% of active samples.

| Case | Total ns/op | Acquisition/KDE | Model build/value match | History split | Other |
|---|---:|---:|---:|---:|---:|
| Parzen full, 1k × 4 | 1,546,696 | 831,503 | 631,825 | — | 83,366 |
| Parzen bounded, 1k × 4 | 841,834 | 387,748 | 284,624 | 163,063 | 6,397 |
| Optimizer, 1k × 4 | 706,349 | 392,659 | 276,111 matching | — | 37,577 |
| Parzen full, 10k × 4 | 17,355,289 | 11,202,839 | 5,373,197 | — | 779,252 |
| Parzen bounded, 10k × 4 | 834,554 | 411,518 | 268,642 | 148,717 | 5,674 |
| Optimizer, 10k × 4 | 6,863,646 | 3,685,091 | 2,804,485 matching | — | 374,068 |
| Parzen full, 1k × 16 | 6,750,192 | 3,809,133 | 2,521,871 | — | 419,186 |
| Parzen bounded, 1k × 16 | 3,527,884 | 1,663,044 | 1,195,599 | 641,369 | 27,870 |
| Optimizer, 1k × 16 | 4,383,279 | 1,500,396 | 2,776,368 matching | — | 106,513 |

For optimizer rows, the first bucket is `sample_tpe_float` and the second is
`find_matching_value`. For Parzen rows, model build includes
`NumericKernels::build` and its sorting and normalization descendants.

## Hypotheses and verdicts

### H1: full-history reconstruction dominates history scaling — partially refuted

Full reconstruction is material, but it is not the dominant large-history
bucket. At 4D/10,000, `ProductMixture::build` was 31% inclusive while
acquisition was 65%. Both are component-scaled and together explain 95.5% of
CPU. Full history filters every trial through `TrialStorage::typed_value` and
allocates an applicable ID vector in `src/sampler.rs:455`; it then extracts
values again in `src/sampler/mixture.rs:53`. Numeric construction allocates and
sorts an index vector and derives bandwidths in
`src/sampler/mixture.rs:345`.

The net full-history root cause is repeated O(history) exact density work plus
O(history log history) rebuild work, not history bookkeeping alone.

### H2: candidate likelihood scoring dominates dimensional scaling — confirmed

Fixed-suggest captures put 98--100% inclusively in
`ProductMixture::log_pdf` (`src/sampler/mixture.rs:205`) and 84--88% in
exponential math called by `logsumexp` (`src/sampler/math.rs:48`).
`TpeSampler::acquire` evaluates both good and bad mixtures for each of 24
candidates at `src/sampler.rs:413`.

The current layout copies all log weights into scratch for every mixture and
candidate (`src/sampler/mixture.rs:223`), then makes a separate component pass
to exponentiate and sum. Exact mixture density requires exponentials, but the
candidate/component loop can be reorganized into reusable contiguous batches
to improve locality and enable vectorization.

The linear candidate parameter lookup at `src/sampler/mixture.rs:225` is not
material in the independent scenario because each estimator has one parameter.
It should be removed positionally as part of a batch layout, not claimed as a
measured standalone win. A grouped 8D capture is required before assigning it a
separate budget.

### H3: transient allocation explains the gap — partially confirmed

At 4D/1,000 and 100 warmed cycles, DHAT reported:

| Backend | Retained after ingest | Allocated bytes/cycle | Peak live bytes |
|---|---:|---:|---:|
| Parzen full | 126,280 | 702,720 | 651,882 |
| Parzen bounded | 179,784 | 437,227 | 469,922 |
| Optimizer | 1,247,168 | 147,659 | 1,633,762 |
| tpe | 65,792 | 483,647 | 200,032 |
| hyperopt | 168,112 | 566 | 195,224 |

Optimizer retained the most heap yet was fastest at this point, so retained
memory is not causal. Parzen allocated substantially more per cycle, and DHAT
stacks point to `ProductMixture::build`, `NumericKernels::build`, and bounded
splitting. However, libc self time was only about 2% in cycle profiles.
Allocation removal is valuable mainly because it also removes initialization,
copying, sorting inputs, and memory traffic; allocator calls alone cannot
explain a 19% or 119% gap.

### H4: bounded splitting wastes work each generation — confirmed

`BoundedHistory::split` (`src/sampler/history.rs:151`) allocates good and bad
vectors, copies top/recent/reservoir entries, sorts them, and truncates them.
`applicable_history` first calls it with a one-element good partition at
`src/sampler.rs:470`; `sample_estimator` calls it again with the true good count
at `src/sampler.rs:367`.

Samply attributed 17.8--19.4% of bounded-cycle CPU to split across all three
profile points. At the curated adverse point that is an approximate 163
microsecond upper bound, larger than the 135 microsecond gap to optimizer.
Eliminating all split cost is unrealistic, but removing duplicate
materialization and repeated allocation is the strongest low-risk first target.

### H5: optimizer's lead is not wrapper overhead — confirmed, with caveats

Optimizer's public wrapper in
`comparison-benchmarks/src/backends/optimizer_backend.rs:98` enqueues exact
fixture values and uses its documented ask/suggest/complete lifecycle. Failed
fixed-history suggestions are explicitly not stored by optimizer 1.0.1, so the
fixed profile does not silently grow completed or failed history.

Its own profile is dominated by sampler internals, not the adapter:

- `find_matching_value` scans per-trial distribution hash maps at
  `optimizer-1.0.1/src/sampler/tpe/sampler.rs:650`.
- Float sampling collects good and bad vectors at
  `optimizer-1.0.1/src/sampler/tpe/sampler.rs:661`.
- KDE construction and 24-candidate density selection occur at
  `optimizer-1.0.1/src/sampler/tpe/common.rs:8`.

At 16 dimensions, matching/hash-map work reached 63% inclusive, allowing
bounded Parzen's contiguous representation to win. Optimizer is therefore a
useful competitor, not a universal performance floor. Its different KDE,
bandwidth, prior, RNG, and density semantics also produced much worse quality
on the selected sphere objective.

## Quality is independent of speed

At 250 total evaluations over 32 deterministic seeds:

| Backend | Median regret | p10 | p90 | Success at regret <= 0.01 |
|---|---:|---:|---:|---:|
| tpe | 0.086789 | 0.028509 | 0.218634 | 2/32 |
| Parzen full | 0.135747 | 0.038941 | 0.404591 | 0/32 |
| Parzen bounded | 0.135747 | 0.038941 | 0.404591 | 0/32 |
| hyperopt | 8.597916 | 2.415701 | 25.496490 | 0/32 |
| optimizer | 23.910660 | 1.909258 | 52.846817 | 0/32 |

At budget 100, median regret was 0.771 for `tpe`, 1.321 for Parzen, 11.170
for Hyperopt, and 23.911 for optimizer. There is no combined speed/quality
score. An optimization that changes the kernel, candidate count, partition,
or approximation policy must re-run all 32 seeds and be treated as an algorithm
change rather than a free performance improvement.

## Recommended refactoring sequence

### 1. Make bounded partitioning single-pass and reusable

Refactor `sample_estimator` so it obtains `seen` and generation without first
materializing retained IDs. Handle startup and flat-categorical checks through a
specialized iterator or cached metadata. Perform only the true good/bad split.

Add estimator-owned split scratch buffers with fixed bounded capacities. A
second experiment may maintain the bad partition in an ordered contiguous
structure, but it must be compared against sorting a reused ~512-entry buffer;
the latter may be simpler and cache-friendlier.

Measured upper bound: 17.8--19.4% of bounded cycles. Expected first-step impact
is lower because one correct split remains. This change alone could plausibly
close much of the 19.2% 4D/1,000 gap without changing sampling semantics.

### 2. Introduce estimator-local typed observation columns

The packed global storage is compact, but full rebuilding repeatedly binary
searches parameter IDs (`src/storage.rs:49`) and decodes the same values. Add
private estimator-local columns containing transformed numeric values or
categorical indices plus trial/rank identity and activation state.

This representation should:

- preserve conditional applicability exactly;
- append/update only affected estimators on completion;
- let full and bounded histories expose ordered indices without decoding;
- keep bounded capacity explicit;
- avoid cloning distributions while building models.

Measured upper bound is the model-build bucket: 31--41% for full and 32--34%
for bounded. Only part is removable because bandwidth and normalization work
remain.

### 3. Reuse model construction storage

Replace build-and-drop vectors for component weights, cumulative weights,
means, sigmas, inverse sigmas, normalizers, coefficients, and sort order with
buffers owned by each model cache. Specialize uniform observation weights so a
scalar observation log-weight plus a prior entry replaces a full repeated
weight vector where exact semantics permit.

Cap bounded-mode buffers at configured limits. Full mode may retain its
high-water capacity and must document that memory tradeoff. Validate with DHAT;
the target is reduced bytes/cycle and initialization work, not merely fewer
allocator calls.

### 4. Batch exact acquisition

Generate the 24 candidates into a positional, estimator-ordered buffer and
score a candidate-by-component matrix for good and bad mixtures. Reuse the
matrix and make component traversal contiguous. Remove `ParamId` searches from
the inner loop. Preserve the exact log-domain density calculation initially;
do not introduce fast-exp approximations or candidate pruning in the first
prototype.

Measured upper bound is 46--65% of cycle CPU and 98--100% of cached-suggest
CPU. Most of that work is mathematically required, so the expected gain cannot
be stated until a prototype shows vectorization or fewer memory passes in a
matching before/after profile.

### 5. Defer incremental full-history maintenance

Numeric bandwidths depend on sorted neighbors. A new observation changes the
inserted point and adjacent bandwidths, but gamma boundary movement and some
weight strategies can change more state. Maintain sorted typed columns first,
then measure whether local updates avoid enough of the 31--41% build bucket to
justify the additional invariants. Do not begin with a complex incremental KDE.

### 6. Profile integer and grouped paths separately

The integer result is adverse, but its discrete Gaussian cell-mass branch calls
CDF/erfc work at `src/sampler/mixture.rs:434`; no integer CPU profile currently
quantifies it. Likewise, positional lookup matters only for grouped estimators,
not the independent dimensional envelope. Capture those two workloads before
specializing either path.

## Public API verdict

No public API change is currently warranted. The first four targets can be
implemented behind the existing `Study` and sampler lifecycle. A prepared or
frozen model handle, typed public ingestion, or batch-suggestion API should be
considered only if an internal prototype demonstrates a remaining measured
cost that the current API makes unavoidable.

## Validation required for every future change

1. Re-run format, tests, Clippy, docs, packaging, and the benchmark smoke test.
2. Preserve deterministic fixture checksums and observation counts.
3. Compare before/after release timing on Balthasar with rotated order.
4. Capture matching Samply profiles and require the targeted bucket and active
   sample count to move in the predicted direction.
5. Run DHAT when the change targets storage or allocation.
6. Re-run all 32 quality seeds and report convergence independently.
7. Reject a claimed win when wall time moves without corresponding profile
   evidence, or when semantics/quality move without explicit approval.

## Accepted profile checksums

Each row is `profile SHA-256 / symbol-sidecar SHA-256`.

```text
parzen-full-fixed
a8d0c505ec50920b9ac9bbfb85b8da27d348a4cec238c19ab3a49eb0ca5abddd / ba14113b4426ce6546561107c1af0947c87a61a332387d75a3a50cee3751db7e
parzen-full-cycle
a0082f5f0e24f0a359253c7138d22b5cfa784b71c69b1679cd1b593a9ed0ef71 / 4bb69fc08f460ca19a46a47ccc3ee69dd709a7b255e34c0b9f4c5c57b1b7c0c4
parzen-bounded-fixed
fc18439f4c7cc7aec3a4ef7391bfc0150352ff26d62bbeada519356a30001682 / 90515b79d29e95c4d7f9c8b4ed937cb2279eb978e4d30c5f5cd0fb423e708d0a
parzen-bounded-cycle
b4d722f9e4afae273007fbf8aa8a0ef98d92f43effeb47a73b87ac2f49180a58 / 53cac7793656c1eb07e2011b51630cc88e3752a7308873134dc2918f94587fb4
optimizer-fixed
e8ff19fd8942c93b8484ae5b9a6efe01c0f1892564909b85e0dce2a22d6c4ce0 / 715363d9e59225685ec99d5429ba9ffe0ad306558b6ef1f43cb7cb7e44861a97
optimizer-cycle
490b80c8651a56115913ce2aa0172728722f47a0c96ce2c8be331752643af780 / 11cb9c90a0a42fa4cb574d2168a7e4cf44a64fa28722664fad5584cec793a115

parzen-full-fixed-h10000-d4
c86544bf7f3cd71c9137cec75164a48323462cdf62eda3cefbce31a9a388b0fb / 486e704b4eef89453dedd3288c9174f601084f3f84e0dc936da64eea867e2f29
parzen-full-cycle-h10000-d4
502047e2e3c0b94ee551796f5c0d63c6e55c7cbf010400a5379d8b2ec7e0da72 / 0fda103683be0c737b0863c9497f1909b1b635da8bb6aceff99b1ef31d70b9ae
parzen-bounded-fixed-h10000-d4
4d2fef1f6561163c53a1a6ec21b273834d95da5cb65af2a774047ce6abaae2fd / deb2eed456c180a1205ec58ae19d5efde2f5ed719bced7ab30dccd64bfaedba1
parzen-bounded-cycle-h10000-d4
2228c79ce0d41d4053eb66db8d67db9b72040d438a8e2d48e0f16924944ba6da / 625d39b9e5a2493430377ee10b1ba811c9e912e1c8453e8ffb25689944154dc4
optimizer-fixed-h10000-d4
334290c42143e0f36b3f9e6a6c3ff8974167839d704d61734ebc3e7f0cd5424e / 0a6f85e10034997430b3d0f1cc0965b119a0f30759c9485f52ce101ab5bf8793
optimizer-cycle-h10000-d4
23f271c33edab3084e4fbbd04c42c51dd1e67d7568386340f361a8aa2a0b848b / 73f1921a5a5268d4a0730fddcbd75dfb2702c075834bfa9baa01d3639b1f071b

parzen-full-fixed-h1000-d16
291778258ac204afb2f7f0f3e905e63453d07809cf9bc6e6cabda5949f787938 / 8d1c0333f8d5b2a3b138f188b4bf6044f6cc381e40e2b17d7235bbac657f2107
parzen-full-cycle-h1000-d16
4bc7b3fff3f068e317a3c6a1a88f7bdcb3a90a1132f6be44ce8829db2627f3ac / 47e618bd6945d5ef415d2b8e5642dc2d8d4c7c7d1a6dd56bb6c2d8b2e3cccc60
parzen-bounded-fixed-h1000-d16
c80a0bd063ef73bed9ea5cf1cd3ebd301ef0bc0efa0f2bf9fb5120c65a15d5b5 / d541cd1826b9cc3d672a395f73c0d21ab275f7e3327dfe4bcb8d47fa77093c5c
parzen-bounded-cycle-h1000-d16
514811adf8ed146da54c3bbbd0af2e7d8e02e86572970dee53223baa0a86e95f / e2b533279feb2b8b4996cff65adf06cfbb914a6fe2e94a8a0b497d64502e9103
optimizer-fixed-h1000-d16
cf7e1124565704c00d21ccadfd8b3bc31999be9522e442e9f9e24ee54ee064d5 / da2795e8c79ada0a524432b8b4e8030d5c1a71fe63d9bab6e0d3983233bacd85
optimizer-cycle-h1000-d16
063bd1c20f23e0ff38de294084af7c0284394864c1c93f4f222ae9e738270a86 / 0e7f1db9c0118891dbf19045266eb79768b85187b0b7fefdf07bac4fabef9843
```
