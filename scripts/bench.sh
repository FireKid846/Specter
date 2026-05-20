#!/bin/bash
# Run Specter performance benchmarks

set -e
echo "Running Specter Engine benchmarks..."

# Perft benchmarks (move generation speed)
echo ""
echo "=== Perft Benchmarks ==="
cargo test perft_startpos_d3 -p specter-engine --release -- --nocapture

# Search benchmarks
echo ""
echo "=== Search Benchmarks ==="
cargo bench -p specter-engine

echo ""
echo "=== Perft Depth 5 (full validation) ==="
cargo test perft_startpos_depth5 -p specter-engine --release -- --nocapture --ignored

echo "Done."
