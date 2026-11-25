# Benchmarks & Coverage

Measured on **Apple M3 Pro** (`arm64`, 11 cores, 18 GiB RAM, macOS) with `rustc 1.100.0` (`--release`) vs. `Python 3.15.2` (same host) and upstream `stanford-oval/genie-worksheets`.

## 1. Code Coverage (`cargo llvm-cov --workspace --summary-only`)

| File | Region Coverage | Function Coverage | Line Coverage |
| :--- | ---: | ---: | ---: |
| `worksheets-core/src/runtime.rs` | 90.36% | 100.00% | 92.86% |
| `worksheets-core/src/schema.rs` | 77.92% | 85.71% | 67.78% |
| `worksheets-core/src/dialogue.rs` | 61.54% | 60.00% | 70.67% |
| **Workspace Total** *(incl. CLI)* | **68.09%** | **76.00%** | **67.88%** |

## 2. Footprint, Startup & Microbenchmarks

| Metric / Operation | `genie-worksheets-rs` (Rust) | Upstream `genie-worksheets` (Python) | Comparison |
| :--- | ---: | ---: | :--- |
| **Standalone Binary Size** | **1.00 MiB** | Python + `pydantic` + `langchain` + `chainlit` | Zero Python dependency |
| **Cold Startup (5-run median)** | **6.19 ms** | **23.61 ms** (`python3 -c ""` bare stdlib) | **3.81× faster** cold start |
| `GenieRuntime` 3-turn dialogue (1000x) | **2.1995 ms** | Python worksheet state machine | **~2.2 µs** per 3-turn dialogue |

Reproduce locally:
```bash
cargo run --release -p worksheets-core --example bench
cargo llvm-cov --workspace --summary-only
```
