#!/bin/bash
set -euo pipefail

APP_DATA_DIR="/opt/usr/share/tizenclaw"
BUNDLE_DIR="${APP_DATA_DIR}/bundles/skills_secure"
ROOTFS_TAR="${APP_DATA_DIR}/rootfs.tar.gz"
CONTAINER_ID="tizenclaw_skills_secure"

detect_runtime() {
  if [ -x /usr/libexec/tizenclaw/crun ]; then
    echo "/usr/libexec/tizenclaw/crun"
    return
  fi
  if command -v crun >/dev/null 2>&1; then
    echo "crun"
    return
  fi
  if command -v runc >/dev/null 2>&1; then
    echo "runc"
    return
  fi
  echo ""
}

RUNTIME_BIN="$(detect_runtime)"

write_config() {
  cat >"${BUNDLE_DIR}/config.json" <<EOF
{
  "ociVersion": "1.0.2",
  "process": {
    "terminal": false,
    "user": {"uid": 0, "gid": 0},
    "args": ["sh", "-lc", "while true; do sleep 3600; done"],
    "env": [
      "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
      "LD_LIBRARY_PATH=/tizen_libs:/tizen_libs64"
    ],
    "cwd": "/",
    "noNewPrivileges": true,
    "capabilities": {
      "bounding": [],
      "effective": [],
      "inheritable": [],
      "permitted": [],
      "ambient": []
    }
  },
  "root": {
    "path": "rootfs",
    "readonly": true
  },
  "mounts": [
    {
      "destination": "/proc",
      "type": "proc",
      "source": "proc"
    },
    {
      "destination": "/dev",
      "type": "bind",
      "source": "/dev",
      "options": ["rbind", "ro"]
    },
    {
      "destination": "/skills",
      "type": "bind",
      "source": "${APP_DATA_DIR}/skills",
      "options": ["rbind", "ro"]
    },
    {
      "destination": "/tizen_libs",
      "type": "bind",
      "source": "/usr/lib",
      "options": ["rbind", "ro"]
    },
    {
      "destination": "/tizen_libs64",
      "type": "bind",
      "source": "/usr/lib64",
      "options": ["rbind", "ro"]
    },
    {
      "destination": "/var/run/dbus",
      "type": "bind",
      "source": "/var/run/dbus",
      "options": ["rbind", "rw"]
    }
  ],
  "linux": {
    "namespaces": [
      {"type": "mount"},
      {"type": "pid"},
      {"type": "ipc"},
      {"type": "uts"},
      {"type": "network"}
    ],
    "maskedPaths": [
      "/proc/acpi",
      "/proc/kcore",
      "/proc/keys",
      "/proc/latency_stats",
      "/proc/timer_list",
      "/proc/timer_stats",
      "/proc/sched_debug",
      "/sys/firmware"
    ],
    "readonlyPaths": [
      "/proc/asound",
      "/proc/bus",
      "/proc/fs",
      "/proc/irq",
      "/proc/sys",
      "/proc/sysrq-trigger"
    ]
  }
}
EOF
}

prepare_bundle() {
  mkdir -p "${BUNDLE_DIR}/rootfs"
  if [ ! -f "${BUNDLE_DIR}/.extracted" ]; then
    tar -xzf "${ROOTFS_TAR}" -C "${BUNDLE_DIR}/rootfs"
    touch "${BUNDLE_DIR}/.extracted"
  fi
  write_config
}

start_container() {
  if [ -z "${RUNTIME_BIN}" ]; then
    echo "No OCI runtime found (crun/runc)" >&2
    return 1
  fi
  prepare_bundle
  "${RUNTIME_BIN}" delete -f "${CONTAINER_ID}" >/dev/null 2>&1 || true

  local RUNTIME_CMD=""
  local NEEDS_FALLBACK=0
  
  if ! "${RUNTIME_BIN}" run --help 2>&1 | grep -q -- "--cgroup-manager"; then
    if ! command -v runc >/dev/null 2>&1; then
      NEEDS_FALLBACK=1
    fi
  fi

  if [ "${NEEDS_FALLBACK}" = "1" ]; then
    echo "Runtime does not support disabling cgroups. Falling back to chroot with unshare."
    
    mkdir -p "${BUNDLE_DIR}/rootfs/skills" "${BUNDLE_DIR}/rootfs/proc" "${BUNDLE_DIR}/rootfs/dev" "${BUNDLE_DIR}/rootfs/var/run/dbus" "${BUNDLE_DIR}/rootfs/tizen_libs" "${BUNDLE_DIR}/rootfs/tizen_libs64"
    
    # Run in background via nohup and fake a container id pid
    nohup unshare -m /bin/sh -c "
      mount --make-rprivate / || true
      mount -t proc proc \"${BUNDLE_DIR}/rootfs/proc\" || true
      mount --rbind /dev \"${BUNDLE_DIR}/rootfs/dev\" || true
      mount --rbind \"${APP_DATA_DIR}/skills\" \"${BUNDLE_DIR}/rootfs/skills\" || true
      mount --rbind /usr/lib \"${BUNDLE_DIR}/rootfs/tizen_libs\" || true
      mount --rbind /usr/lib64 \"${BUNDLE_DIR}/rootfs/tizen_libs64\" || true
      mount --rbind /var/run/dbus \"${BUNDLE_DIR}/rootfs/var/run/dbus\" || true
      exec chroot \"${BUNDLE_DIR}/rootfs\" /bin/sh -lc \"export LD_LIBRARY_PATH=/tizen_libs:/tizen_libs64; while true; do sleep 3600; done\"
    " </dev/null >/dev/null 2>&1 &
    
    echo $! > "${BUNDLE_DIR}/chroot.pid"
    return 0
  fi

  local CGRP_ARG="--cgroup-manager=disabled"
  (
    cd "${BUNDLE_DIR}"
    "${RUNTIME_BIN}" run ${CGRP_ARG} -d "${CONTAINER_ID}"
  )
}

stop_container() {
  if [ -f "${BUNDLE_DIR}/chroot.pid" ]; then
    local CPID=$(cat "${BUNDLE_DIR}/chroot.pid")
    kill -9 $CPID >/dev/null 2>&1 || true
    rm -f "${BUNDLE_DIR}/chroot.pid"
    umount "${BUNDLE_DIR}/rootfs/skills" >/dev/null 2>&1 || true
    umount "${BUNDLE_DIR}/rootfs/dev" >/dev/null 2>&1 || true
    umount "${BUNDLE_DIR}/rootfs/proc" >/dev/null 2>&1 || true
    echo "Stopped chroot container."
    return 0
  fi

  if [ -z "${RUNTIME_BIN}" ]; then
    return 0
  fi
  "${RUNTIME_BIN}" delete -f "${CONTAINER_ID}" >/dev/null 2>&1 || true
}

status_container() {
  if [ -z "${RUNTIME_BIN}" ]; then
    echo "runtime-missing"
    return 1
  fi
  "${RUNTIME_BIN}" state "${CONTAINER_ID}" >/dev/null 2>&1
}

ACTION="${1:-start}"
case "${ACTION}" in
  start)
    start_container
    ;;
  stop)
    stop_container
    ;;
  restart)
    stop_container
    start_container
    ;;
  status)
    status_container
    ;;
  *)
    echo "Usage: $0 {start|stop|restart|status}" >&2
    exit 2
    ;;
esac
