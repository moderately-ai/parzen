use parzen_comparison_benchmarks::{backends::tpe_backend::TpeBackend, run_backend};

fn main() {
    if let Err(error) = run_backend::<TpeBackend>() {
        eprintln!("bench-tpe: {error}");
        std::process::exit(2);
    }
}
