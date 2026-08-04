// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use parzen_comparison_benchmarks::{backends::tpe_backend::TpeBackend, run_backend};

fn main() {
    if let Err(error) = run_backend::<TpeBackend>() {
        eprintln!("bench-tpe: {error}");
        std::process::exit(2);
    }
}
