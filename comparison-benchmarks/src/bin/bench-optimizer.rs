// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use parzen_comparison_benchmarks::{backends::optimizer_backend::OptimizerBackend, run_backend};

fn main() {
    if let Err(error) = run_backend::<OptimizerBackend>() {
        eprintln!("bench-optimizer: {error}");
        std::process::exit(2);
    }
}
