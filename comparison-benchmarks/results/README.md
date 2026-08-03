# Benchmark results

`baselines/` contains reviewed, machine-labelled JSONL and Markdown reports.
`analysis/` contains question-driven investigations grounded in Samply, DHAT,
and relevant source lines.

Absolute timings are machine-specific. Only same-machine runs under comparable
load and build settings should be compared. Timing, memory, and optimization
quality are separate outcomes; no combined score is used.

Raw runs and profiler artifacts are intentionally ignored. A curated baseline
uses `<date>-<machine>.jsonl` and `<date>-<machine>.md`; its analysis uses
`<date>-parzen-performance.md`.

The 2026-08-03 Balthasar SIMD baseline compares the scalar and runtime-dispatched
continuous-acquisition paths from the same clean production commit. Its companion
analysis records accepted and rejected experiments, quality equivalence, Samply
attribution, counter validation, and why the available data does not support a
conventional hardware roofline.
