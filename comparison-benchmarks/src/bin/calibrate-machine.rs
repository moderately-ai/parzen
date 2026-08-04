// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use serde::Serialize;

const DEFAULT_ARRAY_BYTES: usize = 256 * 1024 * 1024;

#[derive(Serialize)]
struct Calibration {
    schema_version: u32,
    calibration: &'static str,
    requested_seconds: f64,
    elapsed_seconds: f64,
    iterations: u64,
    operations: u64,
    bytes: u64,
    operations_per_second: f64,
    bytes_per_second: f64,
    checksum: f64,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("calibrate-machine: {error}");
        std::process::exit(2);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let calibration = args.next().ok_or_else(usage)?;
    let mut seconds = 10_u64;
    let mut bytes = DEFAULT_ARRAY_BYTES;
    let mut json = false;
    while let Some(flag) = args.next() {
        let mut value = || {
            args.next()
                .ok_or_else(|| format!("missing value for `{flag}`"))
        };
        match flag.as_str() {
            "--seconds" => seconds = value()?.parse()?,
            "--bytes" => bytes = value()?.parse()?,
            "--format" => json = value()? == "json",
            "--help" | "-h" => return Err(usage().into()),
            _ => return Err(format!("unknown argument `{flag}`\n{}", usage()).into()),
        }
    }
    if seconds == 0 {
        return Err("seconds must be positive".into());
    }
    let duration = Duration::from_secs(seconds);
    let result = match calibration.as_str() {
        "fma" => calibrate_fma(duration)?,
        "bandwidth" => calibrate_bandwidth(bytes, duration)?,
        _ => return Err(format!("unknown calibration `{calibration}`\n{}", usage()).into()),
    };
    if json {
        println!("{}", serde_json::to_string(&result)?);
    } else {
        println!(
            "{}: {:.3} GFLOP/s, {:.3} GiB/s ({:.3}s)",
            result.calibration,
            result.operations_per_second / 1e9,
            result.bytes_per_second / 1024_f64.powi(3),
            result.elapsed_seconds,
        );
    }
    Ok(())
}

fn usage() -> String {
    "usage: calibrate-machine <fma|bandwidth> [--seconds N] [--bytes N] [--format json]".into()
}

#[cfg(target_arch = "x86_64")]
#[expect(
    unsafe_code,
    reason = "the checked AVX2/FMA calibration calls one target-feature function"
)]
fn calibrate_fma(duration: Duration) -> Result<Calibration, Box<dyn std::error::Error>> {
    if !std::is_x86_feature_detected!("avx2") || !std::is_x86_feature_detected!("fma") {
        return Err("FMA calibration requires x86-64 AVX2 and FMA".into());
    }
    // Each invocation performs 8 vector FMAs × 4 f64 lanes × 2 FLOPs.
    const FLOPS_PER_ITERATION: u64 = 64;
    const CHUNK: u64 = 1_000_000;
    let started = Instant::now();
    let mut iterations = 0_u64;
    let mut state = [1.0_f64; 8];
    while started.elapsed() < duration {
        // SAFETY: AVX2 and FMA support is checked above. The function has no
        // pointer arguments and confines its vector state to local registers.
        state = unsafe { fma_chunk(state, CHUNK) };
        iterations = iterations.saturating_add(CHUNK);
        black_box(state);
    }
    let elapsed = started.elapsed();
    let operations = iterations.saturating_mul(FLOPS_PER_ITERATION);
    Ok(calibration_result(
        "fma",
        duration,
        elapsed,
        iterations,
        operations,
        0,
        state.into_iter().sum(),
    ))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma")]
#[expect(
    unsafe_code,
    reason = "the harness-only AVX2/FMA calibration stores one vector into a local array"
)]
unsafe fn fma_chunk(state: [f64; 8], iterations: u64) -> [f64; 8] {
    use std::arch::x86_64::{_mm256_fmadd_pd, _mm256_set1_pd, _mm256_storeu_pd};

    let multiplier = _mm256_set1_pd(1.000_000_000_000_000_2);
    let addend = _mm256_set1_pd(0.000_000_000_000_000_1);
    let mut accumulators = state.map(|value| _mm256_set1_pd(value));
    for _ in 0..iterations {
        for accumulator in &mut accumulators {
            *accumulator = _mm256_fmadd_pd(*accumulator, multiplier, addend);
        }
    }
    let mut output = [0.0; 8];
    for (index, accumulator) in accumulators.into_iter().enumerate() {
        let mut lanes = [0.0; 4];
        // SAFETY: `lanes` contains four contiguous f64 values, matching the vector width.
        unsafe { _mm256_storeu_pd(lanes.as_mut_ptr(), accumulator) };
        // All four lanes deliberately carry the same recurrence. Keep one lane
        // so feeding a completed chunk into the next one does not multiply the
        // state by the vector width.
        output[index] = lanes[0];
    }
    output
}

#[cfg(not(target_arch = "x86_64"))]
fn calibrate_fma(_duration: Duration) -> Result<Calibration, Box<dyn std::error::Error>> {
    Err("FMA calibration is only implemented for x86-64".into())
}

fn calibrate_bandwidth(
    requested_bytes: usize,
    duration: Duration,
) -> Result<Calibration, Box<dyn std::error::Error>> {
    let elements = requested_bytes / std::mem::size_of::<f64>();
    if elements == 0 {
        return Err("bandwidth array must contain at least one f64".into());
    }
    let mut output = vec![0.0_f64; elements];
    let left = vec![1.0_f64; elements];
    let right = vec![2.0_f64; elements];
    let started = Instant::now();
    let mut iterations = 0_u64;
    while started.elapsed() < duration {
        for ((output, left), right) in output.iter_mut().zip(&left).zip(&right) {
            *output = 3.0_f64.mul_add(*right, *left);
        }
        iterations = iterations.saturating_add(1);
        black_box(&output);
    }
    let elapsed = started.elapsed();
    let bytes_per_pass = u64::try_from(elements)?.saturating_mul(24);
    let total_bytes = iterations.saturating_mul(bytes_per_pass);
    Ok(calibration_result(
        "bandwidth",
        duration,
        elapsed,
        iterations,
        0,
        total_bytes,
        output.iter().step_by(4096).sum(),
    ))
}

fn calibration_result(
    calibration: &'static str,
    requested: Duration,
    elapsed: Duration,
    iterations: u64,
    operations: u64,
    bytes: u64,
    checksum: f64,
) -> Calibration {
    let elapsed_seconds = elapsed.as_secs_f64();
    Calibration {
        schema_version: 1,
        calibration,
        requested_seconds: requested.as_secs_f64(),
        elapsed_seconds,
        iterations,
        operations,
        bytes,
        operations_per_second: operations as f64 / elapsed_seconds,
        bytes_per_second: bytes as f64 / elapsed_seconds,
        checksum,
    }
}
