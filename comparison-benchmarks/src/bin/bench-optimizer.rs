use parzen_comparison_benchmarks::{backends::optimizer_backend::OptimizerBackend, run_backend};

fn main() {
    if let Err(error) = run_backend::<OptimizerBackend>() {
        eprintln!("bench-optimizer: {error}");
        std::process::exit(2);
    }
}
