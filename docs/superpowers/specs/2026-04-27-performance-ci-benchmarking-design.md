# Design Spec: Performance CI & Benchmarking (Phase 2)

## 🎯 Objective
Establish an automated performance monitoring system that measures the effectiveness of SIMD optimizations compared to scalar fallbacks. This ensures that every Pull Request maintains the sub-nanosecond traversal target and avoids performance regressions.

## 🏗️ Architecture
The system utilizes **Criterion.rs** for high-precision benchmarking and a custom **GitHub Action** workflow to report results directly in Pull Requests.

### Data Flow
1. **Trigger:** PR created or updated.
2. **Execution:** `cargo bench -- --output-format bencher` (or raw JSON extraction from `target/criterion`).
3. **Analysis:** A script parses JSON results to calculate the `Speedup Factor = Scalar_Time / SIMD_Time`.
4. **Reporting:** A GitHub Bot posts a Markdown table with results and alerts if the factor drops below a baseline (e.g., 10x).

## 🧩 Components

### 1. GitHub Workflow (`.github/workflows/performance.yml`)
- Environment: Ubuntu Latest (Nightly Rust).
- Tasks: Dependency caching, running benchmarks, and executing the analyzer.

### 2. Result Analyzer (`scripts/analyze_bench.py`)
- Purpose: Aggregates Criterion JSON data.
- Logic:
    - Extract `mean.estimate` for each benchmark group.
    - Match `Scalar` and `SIMD` pairs for the same workload size.
    - Generate Markdown table string.

### 3. Benchmarking Suite (`benches/traversal_bench.rs`)
- Enhancements:
    - Standardized naming convention for benchmarks: `{Method} {Size}` (e.g., `Scalar 1M`, `SIMD 1M`).
    - Fixed sample size (e.g., 100) to balance CI time and accuracy.

## ⚠️ Stability & Performance
- **Soft Alerts:** Regressions will be reported as comments, not as build failures, to prevent blocking development due to noisy cloud runners.
- **Warm-up:** Benchmarks will include a warm-up phase to minimize cache-miss variance.

## 🧪 Success Criteria
- [ ] PRs automatically receive a performance report.
- [ ] SIMD speedup is clearly visible and measurable across different commit versions.
- [ ] No significant overhead added to the overall CI pipeline (> 5 mins).
