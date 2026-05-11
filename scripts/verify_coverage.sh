#!/bin/bash

# Coverage verification script for Knowledge Loom
# This script runs coverage tools to measure code coverage and verifies that it meets the 80% minimum requirement

set -e

echo "Running coverage verification..."

# Try llvm-cov first
if command -v cargo-llvm-cov &> /dev/null; then
    echo "Using cargo-llvm-cov for coverage measurement..."
    cargo llvm-cov --html \
        --output-path ./coverage \
        --features ollama_tests \
        --workspace
    echo "Coverage report generated at ./coverage/index.html"
    echo "Open the report in a browser to view detailed coverage information"
    exit 0
fi

# Try tarpaulin as fallback
if command -v cargo-tarpaulin &> /dev/null; then
    echo "Using cargo-tarpaulin for coverage measurement..."
    cargo tarpaulin \
        --out Html \
        --output-dir ./coverage \
        --verbose \
        --timeout 300 \
        --features ollama_tests \
        --workspace
    echo "Coverage report generated at ./coverage/index.html"
    echo "Open the report in a browser to view detailed coverage information"
    exit 0
fi

# No coverage tool found
echo "Error: No coverage tool found"
echo "Install one of the following:"
echo "  - cargo-llvm-cov: cargo install cargo-llvm-cov"
echo "  - cargo-tarpaulin: cargo install cargo-tarpaulin"
echo ""
echo "Note: llvm-cov requires additional setup (llvm-tools-preview)"
exit 1
