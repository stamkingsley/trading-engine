#!/bin/bash
set -e

echo "Running trading engine benchmarks..."

# 确保项目已构建
if [[ ! -f "./target/release/trading-engine" ]]; then
    echo "Building project first..."
    cargo build --release
fi

# 运行基准测试
echo "Running Criterion benchmarks..."
cargo bench

# 运行负载测试
echo "Running load test..."
RUST_LOG=warn ./target/release/trading-engine --benchmark --duration 60 --events-per-second 100000

# 生成报告
echo "Generating performance report..."
cargo bench -- --output-format html

echo ""
echo "Benchmarks completed!"
echo "Results are available in:"
echo "  - target/criterion/ (detailed benchmark results)"
echo "  - benchmark-report.html (summary report)"