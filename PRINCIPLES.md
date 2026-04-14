# Engineering Principles

These principles guide the development and maintenance of `cntm-graph`.

## 🛠️ Core Architecture
- **KISS:** Our primary architectural guideline to ensure code remains clean and understandable.
- **Zero-Copy Performance:** Secondary principle focusing on the specific performance and safety needs of the `Rust, Shared Memory (SHM), FlatBuffers, SIMD (AVX-512), Mojo FFI` stack.

## ⚖️ Quality Standards
1. **Uncompromising Safety:** Every line of code must prioritize data integrity and memory safety.
2. **Predictable Performance:** Zero-cost abstractions are preferred over convenience if performance is impacted.
3. **Comprehensive Testing:** No feature is complete without an automated test suite runnable via `cargo test`.

## 🤝 Collaborative Values
- **Explicit over Implicit:** Code should be self-documenting and intent should be clear.
- **Incremental Excellence:** We value small, high-quality PRs over massive, complex changes.
