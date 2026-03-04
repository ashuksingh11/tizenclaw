#!/bin/bash
# TizenClaw RootFS Builder Script
# This script builds the Alpine-based Docker image and extracts its filesystem
# to a tar.gz package that will be deployed with the Tizen RPM.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="$PROJECT_ROOT/data"
ROOTFS_TAR="$OUTPUT_DIR/rootfs.tar.gz"
IMAGE_NAME="tizenclaw-rootfs-base"
CONTAINER_NAME="tc-rootfs-extractor"

echo "[1/3] Building Docker image for TizenClaw RootFS..."
sudo docker build -t "$IMAGE_NAME" "$SCRIPT_DIR"

echo "[2/3] Extracting RootFS from image..."
# Create a dummy container to export its filesystem
sudo docker create --name "$CONTAINER_NAME" "$IMAGE_NAME" /bin/true

# Ensure output directory exists (data/ is packaged in RPM %files)
mkdir -p "$OUTPUT_DIR"

# Export the filesystem and compress to tar.gz directly
echo "Exporting and compressing to $ROOTFS_TAR..."
sudo docker export "$CONTAINER_NAME" | gzip > "$ROOTFS_TAR"

echo "[3/3] Cleaning up temporary Docker container..."
sudo docker rm "$CONTAINER_NAME"

echo "✅ RootFS generation complete: $ROOTFS_TAR"
echo "File size: $(du -sh "$ROOTFS_TAR" | cut -f1)"
