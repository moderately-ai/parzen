# Cross-crate TPE benchmarks

This unpublished crate compares Parzen with `tpe 0.3.1`, `hyperopt 0.0.17`, and
`optimizer 1.0.1`. It is a diagnostic tool, not a leaderboard. Unsupported or
non-equivalent cases are recorded explicitly, and results are retained whether
Parzen wins or loses.

The crate is intentionally outside the root package and CI. It inherits the
repository's Rust 1.89.0 toolchain, owns its lockfile and target directory, and
must never be published.

## Build and smoke test

Build every executable before measuring, then invoke the compiled driver:

```bash
cargo build --manifest-path comparison-benchmarks/Cargo.toml --release --bins
comparison-benchmarks/target/release/compare smoke \
  --machine-label my-machine \
  --output comparison-benchmarks/results/raw/smoke.jsonl
```

The driver runs backends serially and rotates their order between rounds. It
never invokes `cargo run`. Before a real suite, characterize every scenario and
operation once without calibration:

```bash
comparison-benchmarks/target/release/compare characterize \
  --machine-label my-machine \
  --output comparison-benchmarks/results/raw/characterize.jsonl
```

Available suites are `smoke`, `characterize`, `timing`, `scaling`, `quality`,
`memory`, and `full`. Use `--backend all|NAME[,NAME...]`, where a name is
`parzen`, `parzen/full`, `parzen/bounded`, `tpe`, `hyperopt`, or `optimizer`,
`--rounds N`, `--samples N`, `--warmup N`, `--calibration-ms N`,
`--timeout-seconds N`, `--quality-seeds N`, `--machine-label LABEL`, and
`--output PATH`. Use `--scenario NAME`, `--operation NAME`, `--history N`, and
`--dimensions N` to run a focused subset without changing the suite
definitions. The driver
prints the case count, backend invocation count,
calibrated sampling-time floor, quality evaluation count, and memory observation
count before starting. Characterization limits each child to 10 seconds;
ordinary suites default to 120 seconds per child and can override that bound
explicitly. Completed JSONL records are flushed after every child invocation.
It captures static toolchain, repository, machine, and load preflight data once
per suite and passes that snapshot to each isolated backend process. Children
refresh their timestamp and CPU affinity without repeatedly launching diagnostic
subprocesses that would add wall time and perturb the host.

Regenerate a report deterministically from existing JSONL:

```bash
comparison-benchmarks/target/release/compare report results.jsonl --output results.md
```

Routine timing uses `Instant`, one warmup, calibration to at least 100 ms, five
internal samples, and three rotated comparison rounds by default. A curated
checkpoint can request the more expensive protocol explicitly with
`--warmup 3 --calibration-ms 250 --samples 10 --rounds 8` after characterization
shows that its cost is justified. The primary number is the minimum ns/op as a
noise-floor estimate; median, mean, standard deviation, p90, p95, throughput,
raw samples, and round wins remain available.
Fixture construction and fixed-history adapter setup are outside the timed
`cold-suggest`, `suggest`, `update`, and `cycle` loops. Each state-growing sample
starts from a newly constructed adapter with identical fixture history.

`ingest` and `cold-suggest` are intentionally not auto-batched. Ingest measures
one complete fixture history per sample. Cold suggestion measures one first
guided suggestion per sample because batching it would rebuild and ingest an
untimed history for every measured operation. Automatic update and cycle
calibration is capped at 100 operations so a sample does not silently turn a
fixed-history question into a materially different, ever-growing history.

Quality is independent of timing. Each run starts with the same deterministic
ten-point design. Exploratory suites default to the first 8 of 32 checked-in
seeds; a curated quality run must request `--quality-seeds 32`. Reports include regret,
success rate, thresholds, and the complete best-so-far curve. There is no
combined speed/quality score.

## Semantic scope

Every record includes the backend's material semantics. These implementations
differ in kernel and bandwidth policy, gamma rounding, priors, categorical
encoding, RNG, history retention, and wrapper cost.

The mixed fixture includes categorical, float, and discrete integer parameters.
Although `tpe` can model the first two independently, version 0.3.1 has no
equivalent discrete integer model, so this harness marks that whole case
unsupported instead of rounding a continuous suggestion and calling it
equivalent. Correlated numeric cases compare only Parzen's explicit grouped
model with `optimizer`'s multivariate TPE. Correlated mixed categorical cases
are non-comparative because no selected competitor has an equivalent joint
categorical model.

## Memory

Memory binaries must be built separately; do not use them for timing:

```bash
CARGO_TARGET_DIR=comparison-benchmarks/target-memory \
cargo build --manifest-path comparison-benchmarks/Cargo.toml \
  --release --features dhat-heap --bins
comparison-benchmarks/target-memory/release/compare memory \
  --machine-label my-machine \
  --output comparison-benchmarks/results/raw/memory.jsonl
```

The `full` command runs ordinary timing/quality binaries and takes the separate
memory build explicitly:

```bash
comparison-benchmarks/target/release/compare full \
  --memory-bin-dir comparison-benchmarks/target-memory/release \
  --machine-label my-machine \
  --output comparison-benchmarks/results/raw/full.jsonl
```

The driver rejects timing suites from a `dhat-heap` build.

Memory records distinguish retained bytes while the ingested optimizer is held,
the first-cycle warmup allocation, warmed-cycle churn, and live bytes at region
end. This follows Squonk's transient-versus-retained convention: live state is
sampled before it is dropped, rather than inferred from cumulative allocation.
DHAT profiles are written to the system
temporary directory and their paths are recorded in JSON. On macOS, peak RSS
comes from `getrusage`; `/usr/bin/time -l -- <binary> ...` is an independent
cross-check. The memory suite visits the five history sizes in a fixed
non-monotonic order to reduce order bias. DHAT retained bytes are the primary
in-process retention measure; a single peak-RSS value includes process and
allocator overhead and must not be presented as exact heap retention. A future
RSS slope should follow Squonk's external-controller approach with randomized
counts and explicit linear-fit quality gates.

## Quiet-machine protocol

Before a curated run, inspect:

```bash
uptime
ps -Ao pcpu,pid,comm -r
```

On macOS, also check screen saver, wallpaper, and video compositor activity.
Do not benchmark during builds, agent work, or another timed workload. Do not
run backends concurrently and do not use blind sleeps as a noise-control method.
The executable records a short load snapshot, but the operator remains
responsible for deciding whether the machine is quiet.

## Samply profiling

Profiling builds preserve full debug information:

```bash
cargo build \
  --manifest-path comparison-benchmarks/Cargo.toml \
  --profile profiling \
  --bins
```

Profile the compiled executable directly, preserving the symbol sidecar and all
threads:

```bash
samply record \
  --save-only \
  --unstable-presymbolicate \
  --rate 4000 \
  -o /tmp/parzen.profile.json.gz \
  -- comparison-benchmarks/target/profiling/bench-parzen \
  --scenario independent-float \
  --operation profile \
  --profile-workload cycle \
  --history 10000 \
  --dimensions 16 \
  --profile-seconds 30
```

Analyze it headlessly with the read-only harness-tools workflow:

```bash
python3 \
  ~/workspace/github.com/tomsanbear/harness-tools/skills/samply-profiling/scripts/analyze_profile.py \
  /tmp/parzen.profile.json.gz
```

Profiled wall time is never benchmark evidence. Separate idle from active CPU
samples, inspect self and inclusive attribution, and read the source behind
every material Parzen frame before making an optimization claim.

## Hardware counters

`calibrate-machine` provides an AVX2/FMA throughput probe and a single-threaded
STREAM-style triad. Build only this calibration binary with native CPU code
generation; production comparison binaries retain their ordinary release
flags. On Balthasar, `scripts/balthasar-counters.sh` records five separate
repetitions per counter group, pins targets to CPU 7, audits the temporary
`kernel.perf_event_paranoid` change, and restores its exact original value.

Treat counter-run wall time as metadata. Call the resulting chart a hardware
roofline only after retired FLOPs agree with the FMA calibration within 20%,
the traffic counter agrees with the known triad traffic within 25%, event
running time is at least 95%, coefficients of variation are at most 10%, and
CPU migrations are zero. Otherwise report the validated IPC/cache/traffic
proxies and empirical runtime envelope without a roofline label.

Use `--profile-workload fixed-suggest` to warm the backend once and repeatedly
exercise suggestion plus public abort/reset against an unchanged history. Use
`--profile-workload cycle` (the default) for the state-growing public
suggest/evaluate/complete lifecycle. Profile records include starting and ending
observation counts so a fixed-history capture cannot silently become a growing
one.

## Curating evidence

Raw JSONL, Samply profiles and sidecars, DHAT profiles, Instruments recordings,
flamegraphs, and temporary reports stay untracked. After a reviewed quiet run,
copy only the curated JSONL, generated Markdown, and source-grounded analysis to
`results/baselines/` and `results/analysis/`.
