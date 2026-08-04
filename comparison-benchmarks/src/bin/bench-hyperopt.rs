// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use parzen_comparison_benchmarks::{backends::hyperopt_backend::HyperoptBackend, run_backend};

fn main() {
    if let Err(error) = run_backend::<HyperoptBackend>() {
        eprintln!("bench-hyperopt: {error}");
        std::process::exit(2);
    }
}
