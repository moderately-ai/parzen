//! Manual, process-isolated comparison harness for Rust TPE implementations.

pub mod adapter;
pub mod backends;
pub mod cli;
pub mod fixtures;
pub mod measurement;
pub mod objectives;
pub mod output;
pub mod report;
pub mod scenarios;

use std::error::Error;

pub type HarnessError = Box<dyn Error + Send + Sync + 'static>;
pub type HarnessResult<T> = Result<T, HarnessError>;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOCATOR: dhat::Alloc = dhat::Alloc;

pub fn run_backend<B: adapter::Backend>() -> HarnessResult<()> {
    let cli = cli::BackendCli::parse(std::env::args().skip(1))?;
    let record = measurement::execute::<B>(&cli)?;
    output::write_record(&record, cli.format)?;
    Ok(())
}
