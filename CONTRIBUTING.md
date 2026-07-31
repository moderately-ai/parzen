# Contributing

Use Rust 1.88 or newer. Before opening a pull request, run formatting, Clippy, nextest, rustdoc
with warnings denied, `cargo deny check`, and `cargo publish --dry-run`.

Algorithm changes must include deterministic tests with explicit seeds.
