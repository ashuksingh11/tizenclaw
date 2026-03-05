#!/bin/bash
# TizenClaw: Build Alpine Linux RootFS tarball with Python & Node.js

set -e

ALPINE_VERSION="3.20.3"
ARCH=$(uname -m)

if [ "$ARCH" = "x86_64" ]; then
    ALPINE_ARCH="x86_64"
elif [ "$ARCH" = "aarch64" ]; then
    ALPINE_ARCH="aarch64"
elif [ "$ARCH" = "armv7l" ]; then
    ALPINE_ARCH="armv7"
else
    echo "Unsupported architecture: $ARCH"
    exit 1
fi

TARBALL_URL="https://dl-cdn.alpinelinux.org/alpine/v3.20/releases/${ALPINE_ARCH}/alpine-minirootfs-${ALPINE_VERSION}-${ALPINE_ARCH}.tar.gz"

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
PROJECT_DIR="${SCRIPT_DIR}/.."
DATA_DIR="${PROJECT_DIR}/data"
ROOTFS_DIR="${DATA_DIR}/rootfs_temp"
OUTPUT_TAR="${DATA_DIR}/rootfs.tar.gz"

echo "Using Project Directory: $PROJECT_DIR"

# Clean up any existing temp dir
if [ -d "$ROOTFS_DIR" ]; then
    sudo rm -rf "$ROOTFS_DIR"
fi
mkdir -p "$ROOTFS_DIR"

echo "Downloading Alpine minirootfs..."
wget -qO "$DATA_DIR/alpine.tar.gz" "$TARBALL_URL"

echo "Extracting minirootfs..."
sudo tar -xf "$DATA_DIR/alpine.tar.gz" -C "$ROOTFS_DIR"
rm "$DATA_DIR/alpine.tar.gz"

echo "Copying DNS resolution file..."
sudo cp /etc/resolv.conf "$ROOTFS_DIR/etc/"

echo "Mounting necessary host filesystems for chroot..."
sudo mount -t proc proc "$ROOTFS_DIR/proc/"
sudo mount -o bind /sys "$ROOTFS_DIR/sys/"
sudo mount -o bind /dev "$ROOTFS_DIR/dev/"

echo "Installing Python 3 and Node.js..."
# Add script to execute inside chroot
cat << 'EOF' > "$ROOTFS_DIR/install.sh"
#!/bin/sh
apk update
apk add --no-cache python3 py3-pip nodejs npm curl ca-certificates bash
rm -rf /var/cache/apk/*
EOF
sudo chmod +x "$ROOTFS_DIR/install.sh"
sudo chroot "$ROOTFS_DIR" /install.sh
sudo rm "$ROOTFS_DIR/install.sh"

echo "Unmounting filesystems..."
sudo umount "$ROOTFS_DIR/proc/"
sudo umount "$ROOTFS_DIR/sys/"
sudo umount "$ROOTFS_DIR/dev/"

echo "Creating final rootfs tarball..."
cd "$ROOTFS_DIR"
sudo tar -czf "$OUTPUT_TAR" *
cd "$PROJECT_DIR"

sudo rm -rf "$ROOTFS_DIR"

echo "Success! RootFS created at: $OUTPUT_TAR"
