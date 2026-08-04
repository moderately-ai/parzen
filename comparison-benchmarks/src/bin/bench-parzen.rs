// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use parzen_comparison_benchmarks::{backends::parzen_backend::ParzenBackend, run_backend};

fn main() {
    if let Err(error) = run_backend::<ParzenBackend>() {
        eprintln!("bench-parzen: {error}");
        std::process::exit(2);
    }
}
