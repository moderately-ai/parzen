use parzen_comparison_benchmarks::{backends::hyperopt_backend::HyperoptBackend, run_backend};

fn main() {
    if let Err(error) = run_backend::<HyperoptBackend>() {
        eprintln!("bench-hyperopt: {error}");
        std::process::exit(2);
    }
}
