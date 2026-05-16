# Design Spec: Billion-scale Stability & Stress Test (Phase 3)

## 🎯 Objective
Verify the stability, reliability, and performance of the `cntm-graph` engine when operating at a massive scale (1,000,000,000+ nodes). The primary focus is on managing memory-mapped files that exceed physical RAM size and analyzing the impact of Page Faults on traversal latency.

## 🏗️ Architecture
The stress test is implemented as a standalone high-performance binary that bypasses standard unit test overhead to maximize throughput.

### 1. Data Generation (The "Billion" Build)
- **Strategy:** Incremental node insertion into a dedicated `stress_test.bin` file.
- **Scale:** 1 Billion nodes with minimal metadata to simulate a ~40GB to 60GB footprint.
- **Resource Management:** Monitoring disk I/O and growth rate during initialization.

### 2. Random Access Probe (The "Pressure" Test)
- **Mechanism:** Generate 1 Million random node indices using a fast RNG (e.g., `SmallRng`).
- **Access Patterns:**
    - **Uniform Random:** Accesses nodes across the entire 60GB range to force OS page swapping.
    - **Locality Stress:** Accesses "clusters" of nodes to see how effective the OS Page Cache is at managing DOD layouts.
- **Measurement:** High-resolution timers (nanoseconds) to capture individual node retrieval latency.

### 3. Monitoring & Metrics
- **Latency Distribution:** Report Mean, P50, P90, and P99 latency.
- **Page Fault Tracking:** Integration with system tools (e.g., `/proc/self/stat`) to correlate latency spikes with OS page faults.
- **Integrity Check:** Randomly verify node data (IDs/Weights) during access to ensure no corruption occurred during massive file growth.

## 🧩 Components
- `examples/stress_test_1b.rs`: The main test runner.
- `scripts/monitor_resources.sh`: A sidecar script to track RAM/Swap usage during the test.

## ⚠️ Hardware Requirements
- **Disk Space:** Minimum 100GB of free space (SSD recommended).
- **RAM:** The test is designed to run on systems with any RAM size (e.g., 8GB, 16GB, 32GB) to test "Out-of-Core" performance.

## 🧪 Success Criteria
- [ ] System completes 1 Billion node insertions without a crash or OOM.
- [ ] Random access P99 latency remains within acceptable "SSD Bound" limits (typically < 100µs).
- [ ] No data corruption detected across the entire billion-node address space.
