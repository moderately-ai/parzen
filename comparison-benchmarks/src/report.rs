use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
};

use crate::{
    HarnessResult,
    cli::ProfileWorkload,
    output::{BenchmarkRecord, SCHEMA_VERSION},
    scenarios::ParzenHistory,
};

pub fn read_jsonl(path: &Path) -> HarnessResult<Vec<BenchmarkRecord>> {
    let input = BufReader::new(File::open(path)?);
    input
        .lines()
        .enumerate()
        .map(|(index, line)| {
            let line = line?;
            let record: BenchmarkRecord = serde_json::from_str(&line)
                .map_err(|error| format!("invalid JSONL record on line {}: {error}", index + 1))?;
            if record.schema_version != SCHEMA_VERSION {
                return Err(format!(
                    "unsupported JSONL schema version {} on line {}; expected {}",
                    record.schema_version,
                    index + 1,
                    SCHEMA_VERSION
                )
                .into());
            }
            Ok(record)
        })
        .collect()
}

pub fn write_markdown(records: &[BenchmarkRecord], mut output: impl Write) -> HarnessResult<()> {
    writeln!(output, "# TPE comparison results")?;
    writeln!(output)?;
    writeln!(
        output,
        "> Absolute timings are machine-specific. Compare only runs captured on the same machine. Timing and quality are reported independently; no combined score is calculated."
    )?;
    let mut grouped =
        BTreeMap::<(String, String, String, usize, usize, usize), Vec<&BenchmarkRecord>>::new();
    for record in records {
        grouped
            .entry((
                record.scenario.to_string(),
                record.operation.to_string(),
                record
                    .profile_workload
                    .map_or_else(String::new, |workload| workload.to_string()),
                record.config.history,
                record.config.dimensions,
                record.config.budget,
            ))
            .or_default()
            .push(record);
    }
    for ((scenario, operation, profile_workload, history, dimensions, budget), group) in grouped {
        let parzen_supported = group.iter().any(|record| {
            record.backend == "parzen" && record.supported && record.execution_error.is_none()
        });
        let competitor_supported = group.iter().any(|record| {
            record.backend != "parzen" && record.supported && record.execution_error.is_none()
        });
        if !parzen_supported || !competitor_supported {
            continue;
        }
        writeln!(output)?;
        writeln!(output, "## {scenario} / {operation}")?;
        writeln!(output)?;
        if !profile_workload.is_empty() {
            writeln!(output, "Profile workload: `{profile_workload}`.")?;
            writeln!(output)?;
        }
        writeln!(
            output,
            "History: {history}; dimensions: {dimensions}; budget: {budget}."
        )?;
        writeln!(output)?;
        if operation == "profile" {
            write_profile_table(&mut output, &group)?;
        } else if operation == "quality" {
            write_quality_table(&mut output, &group)?;
        } else if operation == "memory" {
            write_memory_table(&mut output, &group)?;
        } else {
            write_timing_table(&mut output, &group)?;
        }
    }
    Ok(())
}

fn write_profile_table(output: &mut impl Write, records: &[&BenchmarkRecord]) -> HarnessResult<()> {
    writeln!(
        output,
        "| Backend | Status | Workload | Operations | Start observations | End observations | Profile seconds |"
    )?;
    writeln!(output, "|---|---|---|---:|---:|---:|---:|")?;
    for (backend, group) in records_by_backend(records) {
        let record = group
            .iter()
            .find(|record| record.supported && record.execution_error.is_none());
        if let Some(record) = record {
            let workload = record.profile_workload.unwrap_or(ProfileWorkload::Cycle);
            writeln!(
                output,
                "| {backend} | supported | {workload} | {} | {} | {} | {:.3} |",
                record.profile_operations.unwrap_or(0),
                record.profile_start_observations.unwrap_or(0),
                record.profile_end_observations.unwrap_or(0),
                record.profile_wall_seconds.unwrap_or(0.0),
            )?;
        } else {
            let status = record_failure_status(&group);
            writeln!(output, "| {backend} | {status} | — | — | — | — | — |")?;
        }
    }
    Ok(())
}

fn records_by_backend<'a>(
    records: &[&'a BenchmarkRecord],
) -> BTreeMap<String, Vec<&'a BenchmarkRecord>> {
    let mut grouped = BTreeMap::new();
    for record in records {
        grouped
            .entry(backend_label(record))
            .or_insert_with(Vec::new)
            .push(*record);
    }
    grouped
}

fn backend_label(record: &BenchmarkRecord) -> String {
    if record.backend == "parzen" {
        match record.config.parzen_history {
            ParzenHistory::Full => "parzen/full".into(),
            ParzenHistory::Bounded => "parzen/bounded".into(),
        }
    } else {
        record.backend.clone()
    }
}

fn write_timing_table(output: &mut impl Write, records: &[&BenchmarkRecord]) -> HarnessResult<()> {
    writeln!(
        output,
        "| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |"
    )?;
    writeln!(output, "|---|---|---:|---:|---:|---:|---:|")?;
    let wins = winner_counts(records);
    let rounds = records
        .iter()
        .filter_map(|record| record.comparison_round)
        .max()
        .map_or(0, |round| round + 1);
    for (backend, group) in records_by_backend(records) {
        let supported = group
            .iter()
            .filter(|record| record.supported && record.execution_error.is_none())
            .copied()
            .collect::<Vec<_>>();
        if supported.is_empty() {
            let status = record_failure_status(&group);
            writeln!(output, "| {backend} | {status} | — | — | — | — | — |")?;
            continue;
        }
        let min = supported
            .iter()
            .filter_map(|record| record.timing.as_ref().map(|stats| stats.min_ns))
            .min_by(f64::total_cmp)
            .unwrap_or(f64::NAN);
        let mut raw = supported
            .iter()
            .flat_map(|record| {
                record
                    .timing
                    .iter()
                    .flat_map(|stats| stats.raw_ns_per_operation.iter().copied())
            })
            .collect::<Vec<_>>();
        raw.sort_by(f64::total_cmp);
        let median = quantile(&raw, 0.50);
        let p95 = quantile(&raw, 0.95);
        let throughput = if min > 0.0 { 1e9 / min } else { f64::INFINITY };
        writeln!(
            output,
            "| {backend} | supported | {min:.1} | {median:.1} | {p95:.1} | {throughput:.1} | {}/{} |",
            wins.get(&backend).copied().unwrap_or(0),
            rounds
        )?;
    }
    Ok(())
}

fn winner_counts(records: &[&BenchmarkRecord]) -> HashMap<String, usize> {
    let mut rounds = BTreeMap::<usize, Vec<&BenchmarkRecord>>::new();
    for record in records.iter().copied().filter(|record| {
        record.supported && record.execution_error.is_none() && record.timing.is_some()
    }) {
        rounds
            .entry(record.comparison_round.unwrap_or(0))
            .or_default()
            .push(record);
    }
    let mut wins = HashMap::new();
    for group in rounds.values() {
        let Some(fastest) = group
            .iter()
            .filter_map(|record| record.timing.as_ref().map(|timing| timing.min_ns))
            .min_by(f64::total_cmp)
        else {
            continue;
        };
        for record in group.iter().filter(|record| {
            record
                .timing
                .as_ref()
                .is_some_and(|timing| timing.min_ns == fastest)
        }) {
            *wins.entry(backend_label(record)).or_insert(0) += 1;
        }
    }
    wins
}

fn write_quality_table(output: &mut impl Write, records: &[&BenchmarkRecord]) -> HarnessResult<()> {
    writeln!(
        output,
        "| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |"
    )?;
    writeln!(output, "|---|---|---:|---:|---:|---:|---:|---:|")?;
    for (backend, group) in records_by_backend(records) {
        let mut quality = group
            .iter()
            .filter(|record| record.supported && record.execution_error.is_none())
            .filter_map(|record| record.quality.as_ref())
            .collect::<Vec<_>>();
        if quality.is_empty() {
            let status = record_failure_status(&group);
            writeln!(output, "| {backend} | {status} | — | — | — | — | — | — |")?;
            continue;
        }
        let mut regrets = quality
            .iter()
            .map(|stats| stats.simple_regret)
            .collect::<Vec<_>>();
        let mut best = quality
            .drain(..)
            .map(|stats| stats.best_objective)
            .collect::<Vec<_>>();
        regrets.sort_by(f64::total_cmp);
        best.sort_by(f64::total_cmp);
        let success =
            regrets.iter().filter(|regret| **regret <= 0.01).count() as f64 / regrets.len() as f64;
        writeln!(
            output,
            "| {backend} | supported | {} | {:.6} | {:.6} | {:.6} | {:.1}% | {:.6} |",
            regrets.len(),
            quantile(&regrets, 0.50),
            quantile(&regrets, 0.10),
            quantile(&regrets, 0.90),
            success * 100.0,
            quantile(&best, 0.50)
        )?;
    }
    Ok(())
}

fn write_memory_table(output: &mut impl Write, records: &[&BenchmarkRecord]) -> HarnessResult<()> {
    writeln!(
        output,
        "| Backend | Status | Retained after ingest | Retained at cycle end | Cycle bytes/op | Peak live bytes | Peak RSS bytes |"
    )?;
    writeln!(output, "|---|---|---:|---:|---:|---:|---:|")?;
    for (backend, group) in records_by_backend(records) {
        let memory = group
            .iter()
            .filter(|record| record.execution_error.is_none())
            .find_map(|record| record.memory.as_ref());
        if let Some(memory) = memory {
            writeln!(
                output,
                "| {backend} | supported | {} | {} | {:.1} | {} | {} |",
                memory.retained_bytes_after_ingest,
                memory.current_bytes,
                memory.bytes_per_operation,
                memory.peak_bytes,
                memory.peak_rss_bytes
            )?;
        } else {
            let status = record_failure_status(&group);
            writeln!(output, "| {backend} | {status} | — | — | — | — | — |")?;
        }
    }
    Ok(())
}

fn record_failure_status(records: &[&BenchmarkRecord]) -> String {
    if let Some(reason) = records
        .iter()
        .find_map(|record| record.execution_error.as_deref())
    {
        return format!("timeout: {reason}");
    }
    records
        .iter()
        .find_map(|record| record.unsupported_reason.as_deref())
        .map_or_else(
            || "no successful record".to_owned(),
            |reason| format!("unsupported: {reason}"),
        )
}

fn quantile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let index = ((sorted.len() - 1) as f64 * q).ceil() as usize;
    sorted[index.min(sorted.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_report_is_valid_markdown() {
        let mut output = Vec::new();
        write_markdown(&[], &mut output).expect("report");
        assert!(
            String::from_utf8(output)
                .expect("utf8")
                .contains("TPE comparison")
        );
    }

    #[test]
    fn quantiles_use_nearest_rank() {
        let values = [1.0, 2.0, 3.0, 4.0];
        assert_eq!(quantile(&values, 0.5), 3.0);
        assert_eq!(quantile(&values, 0.9), 4.0);
    }
}
