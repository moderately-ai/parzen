use std::{hint::black_box, time::Instant};

use crate::{
    HarnessResult,
    adapter::Backend,
    cli::{BackendCli, ProfileWorkload, RunConfig},
    fixtures::{Fixture, checksum_values},
    objectives::{evaluate, optimum},
    output::{BenchmarkRecord, Environment, QualityStats, SCHEMA_VERSION, TimingStats},
    scenarios::Operation,
};

const MAX_CALIBRATION_ITERATIONS: usize = 1_048_576;
const MAX_STATE_GROWING_ITERATIONS: usize = 100;

pub fn execute<B: Backend>(cli: &BackendCli) -> HarnessResult<BenchmarkRecord> {
    let config = cli.config.clone();
    let support = B::support(config.scenario);
    let semantics = B::semantics(&config);
    if !support.supported {
        return Ok(BenchmarkRecord::unsupported(
            B::NAME,
            B::VERSION,
            config,
            semantics,
            support.reason.unwrap_or_else(|| "unsupported".to_owned()),
        ));
    }
    let fixture_count = if matches!(config.operation, Operation::Construct | Operation::Quality) {
        0
    } else {
        config.history
    };
    let fixture = Fixture::generate(
        config.scenario,
        config.dimensions,
        fixture_count,
        config.seed ^ 0xd1b5_4a32_d192_ed03,
    )?;
    let mut record = BenchmarkRecord {
        schema_version: SCHEMA_VERSION,
        backend: B::NAME.to_owned(),
        backend_version: B::VERSION.to_owned(),
        scenario: config.scenario,
        operation: config.operation,
        supported: true,
        unsupported_reason: None,
        execution_error: None,
        config: config.clone(),
        semantics,
        fixture_checksum: fixture.checksum,
        result_checksum: 0,
        observations: 0,
        profile_workload: (config.operation == Operation::Profile)
            .then_some(config.profile_workload),
        profile_start_observations: None,
        profile_end_observations: None,
        profile_operations: None,
        profile_wall_seconds: None,
        comparison_round: None,
        invocation_order: None,
        timing: None,
        quality: None,
        memory: None,
        environment: Environment::capture(&config.machine_label),
    };

    match config.operation {
        Operation::Quality => run_quality::<B>(&config, &mut record)?,
        Operation::Memory => run_memory::<B>(&config, &fixture, &mut record)?,
        Operation::Profile => run_profile::<B>(&config, &fixture, &mut record)?,
        _ => run_timing::<B>(&config, &fixture, &mut record)?,
    }
    Ok(record)
}

fn run_timing<B: Backend>(
    config: &RunConfig,
    fixture: &Fixture,
    record: &mut BenchmarkRecord,
) -> HarnessResult<()> {
    for _ in 0..config.warmup {
        let _ = black_box(run_batch::<B>(config, fixture, 1)?);
    }
    let iterations = if config.iterations > 0 {
        config.iterations
    } else if config.operation.is_batchable() {
        calibrate::<B>(config, fixture)?
    } else {
        1
    };
    let mut raw = Vec::with_capacity(config.samples);
    let mut checksum = 0_u64;
    let mut observations = 0;
    for _ in 0..config.samples {
        let (elapsed, outcome) = measure_batch::<B>(config, fixture, iterations)?;
        if outcome.operations == 0 || elapsed == 0 {
            return Err("timing sample performed no measurable work".into());
        }
        raw.push(elapsed as f64 / outcome.operations as f64);
        checksum = checksum.rotate_left(7) ^ outcome.checksum;
        observations = outcome.observations;
    }
    record.result_checksum = checksum;
    record.observations = observations;
    record.timing = Some(summarize_timing(
        raw,
        operations_for(config, fixture, iterations),
    )?);
    Ok(())
}

fn calibrate<B: Backend>(config: &RunConfig, fixture: &Fixture) -> HarnessResult<usize> {
    let target_ns = config.calibration_duration().as_nanos();
    let max_iterations = if matches!(config.operation, Operation::Update | Operation::Cycle) {
        MAX_STATE_GROWING_ITERATIONS
    } else {
        MAX_CALIBRATION_ITERATIONS
    };
    let mut iterations = 1;
    loop {
        let (elapsed, _) = measure_batch::<B>(config, fixture, iterations)?;
        if elapsed >= target_ns || iterations >= max_iterations {
            return Ok(iterations);
        }
        let scale = target_ns.div_ceil(elapsed.max(1));
        let estimated = iterations.saturating_mul(usize::try_from(scale).unwrap_or(usize::MAX));
        iterations = estimated
            .max(iterations.saturating_mul(2))
            .min(max_iterations);
    }
}

fn operations_for(config: &RunConfig, fixture: &Fixture, iterations: usize) -> usize {
    if config.operation == Operation::Ingest {
        fixture.trials.len().max(1).saturating_mul(iterations)
    } else {
        iterations
    }
}

fn measure_batch<B: Backend>(
    config: &RunConfig,
    fixture: &Fixture,
    iterations: usize,
) -> HarnessResult<(u128, BatchOutcome)> {
    match config.operation {
        Operation::Construct => {
            let started = Instant::now();
            let outcome = black_box(run_batch::<B>(config, fixture, iterations)?);
            Ok((started.elapsed().as_nanos(), outcome))
        }
        Operation::Ingest => {
            let mut elapsed = 0;
            let mut observations = 0;
            for _ in 0..iterations {
                let mut backend = B::create(config)?;
                let started = Instant::now();
                for trial in &fixture.trials {
                    backend.ingest(black_box(trial))?;
                }
                elapsed += started.elapsed().as_nanos();
                observations = backend.observations();
            }
            Ok((
                elapsed,
                BatchOutcome {
                    operations: fixture.trials.len().max(1).saturating_mul(iterations),
                    checksum: fixture.checksum,
                    observations,
                },
            ))
        }
        Operation::ColdSuggest => {
            let mut checksum = 0_u64;
            let mut elapsed = 0_u128;
            let mut observations = 0;
            for _ in 0..iterations {
                let mut backend = B::create(config)?;
                ingest_fixture(&mut backend, fixture)?;
                let started = Instant::now();
                let values = black_box(backend.suggest()?);
                elapsed += started.elapsed().as_nanos();
                checksum = checksum.rotate_left(7) ^ checksum_values(&values);
                backend.abort()?;
                observations = backend.observations();
            }
            Ok((
                elapsed,
                BatchOutcome {
                    operations: iterations,
                    checksum,
                    observations,
                },
            ))
        }
        Operation::Suggest | Operation::Update | Operation::Cycle => {
            let mut backend = B::create(config)?;
            ingest_fixture(&mut backend, fixture)?;
            let update_fixture = if config.operation == Operation::Update {
                Some(Fixture::generate(
                    config.scenario,
                    config.dimensions,
                    iterations,
                    config.seed ^ 0xa076_1d64_78bd_642f,
                )?)
            } else {
                None
            };
            let started = Instant::now();
            let mut checksum = 0_u64;
            match config.operation {
                Operation::Suggest => {
                    for _ in 0..iterations {
                        let values = black_box(backend.suggest()?);
                        checksum = checksum.rotate_left(7) ^ checksum_values(&values);
                        backend.abort()?;
                    }
                }
                Operation::Update => {
                    let Some(updates) = update_fixture.as_ref() else {
                        return Err("internal error: update fixture is missing".into());
                    };
                    for trial in &updates.trials {
                        backend.ingest(black_box(trial))?;
                    }
                    checksum = updates.checksum;
                }
                Operation::Cycle => {
                    for _ in 0..iterations {
                        let values = black_box(backend.suggest()?);
                        let objective = black_box(evaluate(config.scenario, &values)?);
                        checksum = checksum.rotate_left(7)
                            ^ checksum_values(&values)
                            ^ objective.to_bits();
                        backend.complete(objective)?;
                    }
                }
                _ => unreachable!(),
            }
            let elapsed = started.elapsed().as_nanos();
            Ok((
                elapsed,
                BatchOutcome {
                    operations: iterations,
                    checksum,
                    observations: backend.observations(),
                },
            ))
        }
        Operation::Quality | Operation::Memory | Operation::Profile => {
            Err("operation requires a dedicated execution path".into())
        }
    }
}

struct BatchOutcome {
    operations: usize,
    checksum: u64,
    observations: usize,
}

fn run_batch<B: Backend>(
    config: &RunConfig,
    fixture: &Fixture,
    iterations: usize,
) -> HarnessResult<BatchOutcome> {
    match config.operation {
        Operation::Construct => {
            let mut checksum = 0_u64;
            for _ in 0..iterations {
                let backend = black_box(B::create(config)?);
                checksum ^= backend.observations() as u64;
                black_box(backend);
            }
            Ok(BatchOutcome {
                operations: iterations,
                checksum,
                observations: 0,
            })
        }
        Operation::Ingest => {
            let mut backend = B::create(config)?;
            for trial in &fixture.trials {
                backend.ingest(black_box(trial))?;
            }
            Ok(BatchOutcome {
                operations: fixture.trials.len().max(1),
                checksum: fixture.checksum,
                observations: backend.observations(),
            })
        }
        Operation::ColdSuggest => {
            let mut checksum = 0_u64;
            let mut observations = 0;
            for _ in 0..iterations {
                let mut backend = B::create(config)?;
                ingest_fixture(&mut backend, fixture)?;
                let values = black_box(backend.suggest()?);
                checksum ^= checksum_values(&values);
                backend.abort()?;
                observations = backend.observations();
            }
            Ok(BatchOutcome {
                operations: iterations,
                checksum,
                observations,
            })
        }
        Operation::Suggest => {
            let mut backend = B::create(config)?;
            ingest_fixture(&mut backend, fixture)?;
            let mut checksum = 0_u64;
            for _ in 0..iterations {
                let values = black_box(backend.suggest()?);
                checksum = checksum.rotate_left(7) ^ checksum_values(&values);
                backend.abort()?;
            }
            Ok(BatchOutcome {
                operations: iterations,
                checksum,
                observations: backend.observations(),
            })
        }
        Operation::Update => {
            let mut backend = B::create(config)?;
            ingest_fixture(&mut backend, fixture)?;
            let update_fixture = Fixture::generate(
                config.scenario,
                config.dimensions,
                iterations,
                config.seed ^ 0xa076_1d64_78bd_642f,
            )?;
            for trial in &update_fixture.trials {
                backend.ingest(black_box(trial))?;
            }
            Ok(BatchOutcome {
                operations: iterations,
                checksum: update_fixture.checksum,
                observations: backend.observations(),
            })
        }
        Operation::Cycle => {
            let mut backend = B::create(config)?;
            ingest_fixture(&mut backend, fixture)?;
            let mut checksum = 0_u64;
            for _ in 0..iterations {
                let values = black_box(backend.suggest()?);
                let objective = black_box(evaluate(config.scenario, &values)?);
                checksum = checksum.rotate_left(7) ^ checksum_values(&values) ^ objective.to_bits();
                backend.complete(objective)?;
            }
            Ok(BatchOutcome {
                operations: iterations,
                checksum,
                observations: backend.observations(),
            })
        }
        Operation::Quality | Operation::Memory | Operation::Profile => {
            Err("operation requires a dedicated execution path".into())
        }
    }
}

fn ingest_fixture<B: Backend>(backend: &mut B, fixture: &Fixture) -> HarnessResult<()> {
    for trial in &fixture.trials {
        backend.ingest(trial)?;
    }
    if backend.observations() != fixture.trials.len() {
        return Err(format!(
            "backend retained {} observations, expected {}",
            backend.observations(),
            fixture.trials.len()
        )
        .into());
    }
    Ok(())
}

fn run_quality<B: Backend>(config: &RunConfig, record: &mut BenchmarkRecord) -> HarnessResult<()> {
    let startup = Fixture::generate(config.scenario, config.dimensions, 10, config.seed)?;
    let mut backend = B::create(config)?;
    ingest_fixture(&mut backend, &startup)?;
    let mut best = startup
        .trials
        .iter()
        .map(|trial| trial.objective)
        .fold(f64::INFINITY, f64::min);
    let mut curve = startup
        .trials
        .iter()
        .scan(f64::INFINITY, |current, trial| {
            *current = current.min(trial.objective);
            Some(*current)
        })
        .collect::<Vec<_>>();
    let mut checksum = startup.checksum;
    for _ in 10..config.budget {
        let values = black_box(backend.suggest()?);
        let objective = black_box(evaluate(config.scenario, &values)?);
        backend.complete(objective)?;
        best = best.min(objective);
        curve.push(best);
        checksum = checksum.rotate_left(7) ^ checksum_values(&values) ^ objective.to_bits();
    }
    let thresholds = [1.0, 0.1, 0.01, 0.001]
        .into_iter()
        .map(|threshold| {
            let reached = curve
                .iter()
                .position(|value| *value <= threshold)
                .map(|i| i + 1);
            (threshold, reached)
        })
        .collect();
    record.fixture_checksum = startup.checksum;
    record.result_checksum = checksum;
    record.observations = backend.observations();
    record.quality = Some(QualityStats {
        best_objective: best,
        simple_regret: best - optimum(config.scenario),
        best_so_far: curve,
        evaluations_to_thresholds: thresholds,
    });
    Ok(())
}

#[cfg(feature = "dhat-heap")]
fn run_memory<B: Backend>(
    config: &RunConfig,
    fixture: &Fixture,
    record: &mut BenchmarkRecord,
) -> HarnessResult<()> {
    use crate::output::{MemoryStats, peak_rss_bytes};
    let profile_path = std::env::temp_dir().join(format!(
        "parzen-comparison-dhat-{}-{}.json",
        B::NAME,
        std::process::id()
    ));
    let profiler = dhat::Profiler::builder().file_name(&profile_path).build();
    let mut backend = B::create(config)?;
    ingest_fixture(&mut backend, fixture)?;
    let after_ingest = dhat::HeapStats::get();
    let warmup_values = black_box(backend.suggest()?);
    let warmup_objective = black_box(evaluate(config.scenario, &warmup_values)?);
    backend.complete(warmup_objective)?;
    let after_warmup = dhat::HeapStats::get();
    let operations = if config.iterations == 0 {
        100
    } else {
        config.iterations
    };
    let mut checksum = 0_u64;
    for _ in 0..operations {
        let values = black_box(backend.suggest()?);
        let objective = black_box(evaluate(config.scenario, &values)?);
        checksum = checksum.rotate_left(7) ^ checksum_values(&values) ^ objective.to_bits();
        backend.complete(objective)?;
    }
    let held = dhat::HeapStats::get();
    record.result_checksum = checksum;
    record.observations = backend.observations();
    drop(backend);
    let done = dhat::HeapStats::get();
    record.memory = Some(MemoryStats {
        total_blocks: done.total_blocks,
        total_bytes: done.total_bytes,
        current_blocks: held.curr_blocks,
        current_bytes: held.curr_bytes,
        blocks_after_drop: done.curr_blocks,
        bytes_after_drop: done.curr_bytes,
        peak_blocks: done.max_blocks,
        peak_bytes: done.max_bytes,
        bytes_per_operation: held.total_bytes.saturating_sub(after_warmup.total_bytes) as f64
            / operations as f64,
        retained_blocks_after_ingest: after_ingest.curr_blocks,
        retained_bytes_after_ingest: after_ingest.curr_bytes,
        warmup_allocated_blocks: after_warmup
            .total_blocks
            .saturating_sub(after_ingest.total_blocks),
        warmup_allocated_bytes: after_warmup
            .total_bytes
            .saturating_sub(after_ingest.total_bytes),
        cycle_allocated_blocks: held.total_blocks.saturating_sub(after_warmup.total_blocks),
        cycle_allocated_bytes: held.total_bytes.saturating_sub(after_warmup.total_bytes),
        peak_rss_bytes: peak_rss_bytes(),
        heap_profile_path: profile_path.display().to_string(),
    });
    drop(profiler);
    Ok(())
}

#[cfg(not(feature = "dhat-heap"))]
fn run_memory<B: Backend>(
    _config: &RunConfig,
    _fixture: &Fixture,
    _record: &mut BenchmarkRecord,
) -> HarnessResult<()> {
    let _ = std::marker::PhantomData::<B>;
    Err("memory operation requires `--features dhat-heap`".into())
}

fn run_profile<B: Backend>(
    config: &RunConfig,
    fixture: &Fixture,
    record: &mut BenchmarkRecord,
) -> HarnessResult<()> {
    let mut backend = B::create(config)?;
    ingest_fixture(&mut backend, fixture)?;
    if config.profile_workload == ProfileWorkload::FixedSuggest {
        let values = black_box(backend.suggest()?);
        black_box(values);
        backend.abort()?;
    }
    let start_observations = backend.observations();
    let started = Instant::now();
    let mut checksum = config.profile_workload.checksum_tag();
    let mut operations = 0_usize;
    while started.elapsed() < config.profile_duration() {
        match config.profile_workload {
            ProfileWorkload::FixedSuggest => {
                let values = black_box(backend.suggest()?);
                checksum = checksum.rotate_left(7) ^ checksum_values(&values);
                backend.abort()?;
            }
            ProfileWorkload::Cycle => {
                let values = black_box(backend.suggest()?);
                let objective = black_box(evaluate(config.scenario, &values)?);
                checksum = checksum.rotate_left(7) ^ checksum_values(&values) ^ objective.to_bits();
                backend.complete(objective)?;
            }
        }
        operations += 1;
    }
    let end_observations = backend.observations();
    match config.profile_workload {
        ProfileWorkload::FixedSuggest if end_observations != start_observations => {
            return Err("fixed-suggest profile changed the observation count".into());
        }
        ProfileWorkload::Cycle
            if end_observations.saturating_sub(start_observations) != operations =>
        {
            return Err(
                "cycle profile observation growth did not match completed operations".into(),
            );
        }
        _ => {}
    }
    record.result_checksum = checksum;
    record.observations = end_observations;
    record.profile_start_observations = Some(start_observations);
    record.profile_end_observations = Some(end_observations);
    record.profile_operations = Some(operations);
    record.profile_wall_seconds = Some(started.elapsed().as_secs_f64());
    Ok(())
}

fn summarize_timing(raw: Vec<f64>, operations: usize) -> HarnessResult<TimingStats> {
    if raw.is_empty() || raw.iter().any(|value| !value.is_finite() || *value < 0.0) {
        return Err("timing samples must be finite and nonnegative".into());
    }
    let mut sorted = raw.clone();
    sorted.sort_by(f64::total_cmp);
    let mean = raw.iter().sum::<f64>() / raw.len() as f64;
    let variance = raw.iter().map(|value| (value - mean).powi(2)).sum::<f64>() / raw.len() as f64;
    let min = sorted[0];
    Ok(TimingStats {
        operations_per_sample: operations,
        min_ns: min,
        median_ns: percentile(&sorted, 0.5),
        mean_ns: mean,
        stddev_ns: variance.sqrt(),
        p90_ns: percentile(&sorted, 0.9),
        p95_ns: percentile(&sorted, 0.95),
        operations_per_second: if min == 0.0 { f64::INFINITY } else { 1e9 / min },
        raw_ns_per_operation: raw,
    })
}

fn percentile(sorted: &[f64], quantile: f64) -> f64 {
    let index = ((sorted.len() - 1) as f64 * quantile).ceil() as usize;
    sorted[index.min(sorted.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timing_summary_is_stable() {
        let stats = summarize_timing(vec![4.0, 1.0, 3.0, 2.0], 7).expect("valid stats");
        assert_eq!(stats.min_ns, 1.0);
        assert_eq!(stats.median_ns, 3.0);
        assert_eq!(stats.p95_ns, 4.0);
        assert_eq!(stats.operations_per_sample, 7);
    }
}
