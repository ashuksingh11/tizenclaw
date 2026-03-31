#!/bin/bash
# tizenclaw workspace global cleanup utility
# This script forcefully removes unnecessary build artifacts, Cargo caches, and temporary files 
# generated during local WSL execution, ensuring a clean git staging area before commits.

echo "🧹 Initiating TizenClaw Workspace Cleanup..."

# 1. Clean Rust cargo caches
echo "[1/4] Cleaning Rust Cargo targets..."
cargo clean
rm -rf target/
find src -type d -name "target" -exec rm -rf {} +

# 2. Clean Tizen GBS output directories
echo "[2/4] Cleaning GBS RPM build outputs..."
rm -rf RPMS/ SRPMS/ BUILD/ BUILDROOT/
find . -path "./vendor" -prune -o -type f -name "*.rpm" -exec rm -f {} +

# 3. Clean C/C++ build artifacts (legacy/FFI remnants)
echo "[3/4] Cleaning C object file remnants..."
find src -type f -name "*.o" -delete
find src -type f -name "*.a" -delete

# 4. Clean editor swap files & temporaries
echo "[4/4] Cleaning editor temp files..."
find . -type f -name "*~" -delete
find . -type f -name "*.swp" -delete
find . -type d -name ".ruff_cache" -exec rm -rf {} +
find . -type d -name "__pycache__" -exec rm -rf {} +

echo "✅ Cleanup complete. The workspace is pristine for git commit."
