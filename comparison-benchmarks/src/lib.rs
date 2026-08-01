//! Manual, process-isolated comparison harness for Rust TPE implementations.

pub mod adapter;
pub mod backends;
pub mod cli;
pub mod fixtures;
pub mod objectives;
pub mod output;
pub mod scenarios;

use std::error::Error;

pub type HarnessError = Box<dyn Error + Send + Sync + 'static>;
pub type HarnessResult<T> = Result<T, HarnessError>;
