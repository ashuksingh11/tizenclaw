#!/bin/bash
# TizenClaw: Build Alpine Linux RootFS tarball with Python & Node.js
#
# Usage:
#   ./build_rootfs.sh              # Build for the current host arch
#   TARGET_ARCH=aarch64 ./build_rootfs.sh   # Cross-build for aarch64
#   TARGET_ARCH=armv7l  ./build_rootfs.sh   # Cross-build for armv7l
#
# Cross-build downloads the Alpine minirootfs without chroot package
# installation (the container bind-mounts /usr from the host at runtime).

set -e

MOUNTED=false

cleanup() {
    if [ "$MOUNTED" = true ] && [ -n "$ROOTFS_DIR" ]; then
        echo "Cleaning up mounts..."
        sudo umount "$ROOTFS_DIR/proc" 2>/dev/null || true
        sudo umount "$ROOTFS_DIR/sys" 2>/dev/null || true
        sudo umount "$ROOTFS_DIR/dev" 2>/dev/null || true
        MOUNTED=false
    fi
}

trap cleanup EXIT

ALPINE_VERSION="3.20.3"

# Determine target architecture (allow cross-build via TARGET_ARCH env)
ARCH="${TARGET_ARCH:-$(uname -m)}"

# Map system arch → Alpine download arch
case "$ARCH" in
    x86_64)   ALPINE_ARCH="x86_64" ;;
    aarch64)  ALPINE_ARCH="aarch64" ;;
    armv7l|armv7hl) ALPINE_ARCH="armv7" ;;
    *)
        echo "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

TARBALL_URL="https://dl-cdn.alpinelinux.org/alpine/v3.20/releases/${ALPINE_ARCH}/alpine-minirootfs-${ALPINE_VERSION}-${ALPINE_ARCH}.tar.gz"

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
PROJECT_DIR="${SCRIPT_DIR}/.."
DATA_DIR="${PROJECT_DIR}/data"
ROOTFS_DIR="${DATA_DIR}/rootfs_temp"
# Use system arch name for output directory (matches CMake TIZENCLAW_ARCH)
OUTPUT_TAR="${DATA_DIR}/img/${ARCH}/rootfs.tar.gz"
mkdir -p "${DATA_DIR}/img/${ARCH}"

HOST_ARCH=$(uname -m)
CROSS_BUILD=false
if [ "$ARCH" != "$HOST_ARCH" ]; then
    CROSS_BUILD=true
    echo "Cross-building rootfs for ${ARCH} on ${HOST_ARCH} host"
fi

echo "Using Project Directory: $PROJECT_DIR"

# Clean up any existing temp dir (unmount first if needed)
if [ -d "$ROOTFS_DIR" ]; then
    sudo umount "$ROOTFS_DIR/proc" 2>/dev/null || true
    sudo umount "$ROOTFS_DIR/sys" 2>/dev/null || true
    sudo umount "$ROOTFS_DIR/dev" 2>/dev/null || true
    sudo rm -rf "$ROOTFS_DIR"
fi
mkdir -p "$ROOTFS_DIR"

echo "Downloading Alpine minirootfs for ${ARCH} (${ALPINE_ARCH})..."
wget -qO "$DATA_DIR/alpine.tar.gz" "$TARBALL_URL"

echo "Extracting minirootfs..."
sudo tar -xf "$DATA_DIR/alpine.tar.gz" -C "$ROOTFS_DIR"
rm "$DATA_DIR/alpine.tar.gz"

QEMU_BIN=""
if [ "$CROSS_BUILD" = true ]; then
    # Cross-build: use QEMU user-static for chroot into foreign-arch rootfs
    case "$ARCH" in
        aarch64)       QEMU_BIN="qemu-aarch64-static" ;;
        armv7l|armv7hl) QEMU_BIN="qemu-arm-static" ;;
    esac
    QEMU_PATH=$(which "$QEMU_BIN" 2>/dev/null || true)
    if [ -z "$QEMU_PATH" ]; then
        echo "Error: $QEMU_BIN not found. Install qemu-user-static." >&2
        exit 1
    fi
    echo "Cross-build: using $QEMU_BIN for chroot..."
    sudo cp "$QEMU_PATH" "$ROOTFS_DIR/usr/bin/"
fi

echo "Copying DNS resolution file..."
sudo cp /etc/resolv.conf "$ROOTFS_DIR/etc/"

echo "Mounting necessary host filesystems for chroot..."
sudo mount -t proc proc "$ROOTFS_DIR/proc/"
sudo mount -o bind /sys "$ROOTFS_DIR/sys/"
sudo mount -o bind /dev "$ROOTFS_DIR/dev/"
MOUNTED=true

echo "Installing Python 3 and Node.js..."
cat << 'EOF' | sudo tee "$ROOTFS_DIR/install.sh" > /dev/null
#!/bin/sh
apk update
apk add --no-cache python3 py3-pip nodejs npm curl ca-certificates bash
rm -rf /var/cache/apk/*
EOF
sudo chmod +x "$ROOTFS_DIR/install.sh"
sudo chroot "$ROOTFS_DIR" /install.sh
sudo rm "$ROOTFS_DIR/install.sh"

echo "Unmounting filesystems..."
cleanup

# Clean up QEMU binary from rootfs (not needed at runtime)
if [ -n "$QEMU_BIN" ]; then
    sudo rm -f "$ROOTFS_DIR/usr/bin/$QEMU_BIN"
fi

echo "Creating final rootfs tarball..."
cd "$ROOTFS_DIR"
sudo tar -czf "$OUTPUT_TAR" *
cd "$PROJECT_DIR"

sudo rm -rf "$ROOTFS_DIR"

echo "Success! RootFS created at: $OUTPUT_TAR"
