# Contributing

Use Rust 1.89 or newer. Before opening a pull request, run formatting, Clippy, nextest, rustdoc
with warnings denied, `cargo deny check`, and `cargo publish --dry-run`.

Maintainers preparing a release must also follow [RELEASING.md](RELEASING.md).

Algorithm changes must include deterministic tests with explicit seeds.

Run `cargo bench --bench suggestion` for hot-path changes and `cargo test --test quality --release`
for statistical checks. Report cold model-building and warmed suggestion behavior separately.
Sampler changes must report independent, grouped, discrete, suggest-plus-complete scaling, retained
history, and warmed-allocation behavior. Do not add wall-clock assertions to ordinary CI tests.
