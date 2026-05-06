# Performance CI & Benchmarking Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an automated performance reporting system for PRs that calculates the speedup factor between SIMD and Scalar traversal.

**Architecture:** A GitHub Action runs Criterion benchmarks, exports JSON results, and a Python script analyzes these results to post a comparative Markdown table as a PR comment.

**Tech Stack:** Rust (Criterion), Python (Analysis), GitHub Actions.

---

### Task 1: Standardize Benchmarking Names

**Files:**
- Modify: `benches/traversal_bench.rs`

- [ ] **Step 1: Update benchmark names to include workload size and method**

Modify `benches/traversal_bench.rs` to use a consistent naming pattern `{Method} {Size}`.

```rust
// ... existing imports ...

fn criterion_benchmark(c: &mut Criterion) {
    let path = "bench_graph.bin";
    let _ = fs::remove_file(path);

    let node_count = 1_000_000;
    let mut store = GraphStore::new(path, node_count, 10).unwrap();

    for i in 0..node_count {
        let type_id = if i % 100 == 0 { 42 } else { 1 };
        let weight = (i as f32) * 0.0001;
        store.nodes.add_node(i as u64, type_id, weight);
    }

    let mut group = c.benchmark_group("Traversal");
    // Ensure names are parseable by the analyzer
    group.bench_function("Scalar 1M", |b| b.iter(|| scalar_traversal(&store, 42)));
    group.bench_function("SIMD 1M", |b| b.iter(|| store.find_best_weighted_simd(42)));

    group.finish();
    let _ = fs::remove_file(path);
}
// ...
```

- [ ] **Step 2: Verify benchmarks still run**

Run: `cargo bench -- --noop` (to check compilation)
Expected: `Finished bench [optimized] target(s) in ...`

- [ ] **Step 3: Commit**

```bash
git add benches/traversal_bench.rs
git commit -m "perf: standardize benchmark names for CI analysis"
```

---

### Task 2: Create Benchmark Result Analyzer

**Files:**
- Create: `scripts/analyze_bench.py`

- [ ] **Step 1: Write the analyzer script**

Create `scripts/analyze_bench.py` that parses Criterion's `estimates.json` files and calculates speedup.

```python
import os
import json
import glob

def get_mean(path):
    with open(path, 'r') as f:
        data = json.load(f)
        return data['mean']['point_estimate'] # nanoseconds

def main():
    results = {}
    # Criterion saves results in target/criterion/{name}/new/estimates.json
    paths = glob.glob("target/criterion/*/new/estimates.json")

    for p in paths:
        name = p.split('/')[-3]
        results[name] = get_mean(p)

    # Process matches
    output = "### 🚀 Performance Report (SIMD vs Scalar)\n\n"
    output += "| Workload | Scalar Time | SIMD Time | **Speedup** |\n"
    output += "| :--- | :--- | :--- | :--- |\n"

    workloads = set(name.split(' ')[1] for name in results.keys())

    for w in workloads:
        scalar = results.get(f"Scalar {w}")
        simd = results.get(f"SIMD {w}")

        if scalar and simd:
            ratio = scalar / simd
            s_time = f"{scalar/1000:,.2f} µs" if scalar > 1000 else f"{scalar:.2f} ns"
            i_time = f"{simd/1000:,.2f} µs" if simd > 1000 else f"{simd:.2f} ns"
            emoji = "✅" if ratio > 10 else "⚠️"
            output += f"| Traversal ({w}) | {s_time} | {i_time} | {emoji} **{ratio:.2x}** |\n"

    print(output)

if __name__ == "__main__":
    main()
```

- [ ] **Step 2: Test analyzer with dummy data (optional but recommended)**

Create a dummy directory structure and run the script to see if it generates a table.
Expected: A Markdown table printed to stdout.

- [ ] **Step 3: Commit**

```bash
git add scripts/analyze_bench.py
git commit -m "perf: add benchmark result analyzer script"
```

---

### Task 3: Configure GitHub Performance Workflow

**Files:**
- Create: `.github/workflows/performance.yml`

- [ ] **Step 1: Write the workflow YAML**

Create `.github/workflows/performance.yml` to orchestrate the whole process.

```yaml
name: Performance Benchmarking
on:
  pull_request:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Setup Rust (Nightly)
        uses: dtolnay/rust-toolchain@nightly
      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-bench-${{ hashFiles('**/Cargo.lock') }}

      - name: Run Benchmarks
        run: cargo bench -- --output-format bencher

      - name: Analyze Results
        id: analyze
        run: |
          REPORT=$(python3 scripts/analyze_bench.py)
          # Escape newlines for GitHub output
          REPORT="${REPORT//'%'/'%25'}"
          REPORT="${REPORT//$'\n'/'%0A'}"
          REPORT="${REPORT//$'\r'/'%0D'}"
          echo "report=$REPORT" >> $GITHUB_OUTPUT

      - name: Post PR Comment
        uses: mshick/add-pr-comment@v2
        with:
          message: ${{ steps.analyze.outputs.report }}
```

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/performance.yml
git commit -m "ci: add performance benchmarking workflow"
```

---

### Task 4: Final Validation

- [ ] **Step 1: Run complete pipeline locally**

Run: `cargo bench && python3 scripts/analyze_bench.py`
Expected: Output table shows correct values for current machine.

- [ ] **Step 2: Push and check PR**

Push the changes to a new branch and open a test PR.
Expected: GitHub Bot posts the performance table as a comment.
