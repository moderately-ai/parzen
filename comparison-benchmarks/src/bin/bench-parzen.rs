use parzen_comparison_benchmarks::{backends::parzen_backend::ParzenBackend, run_backend};

fn main() {
    if let Err(error) = run_backend::<ParzenBackend>() {
        eprintln!("bench-parzen: {error}");
        std::process::exit(2);
    }
}
