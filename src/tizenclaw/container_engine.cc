#include "container_engine.hh"

#include "../common/logging.hh"
#include <array>
#include <cstdio>
#include <cstdlib>
#include <fstream>
#include <memory>
#include <string>
#include <unistd.h>

namespace tizenclaw {


#ifndef APP_DATA_DIR
#define APP_DATA_DIR "/opt/usr/share/tizenclaw"
#endif

namespace {
constexpr const char* kSkillsContainerId = "tizenclaw_skills_secure";
}

ContainerEngine::ContainerEngine()
    : m_initialized(false),
      m_runtime_bin("crun"),
      m_app_data_dir(APP_DATA_DIR),
      m_skills_dir(BuildPaths("skills")),
      m_bundle_dir(BuildPaths("bundles/skills_secure")),
      m_rootfs_tar(BuildPaths("rootfs.tar.gz")),
      m_container_id(kSkillsContainerId) {
}

ContainerEngine::~ContainerEngine() {
    StopSkillsContainer();
}

bool ContainerEngine::Initialize() {
  if (m_initialized) return true;

  LOG(INFO) << "ContainerEngine Initializing...";

  const char* bundled_crun = "/usr/libexec/tizenclaw/crun";
  if (access(bundled_crun, X_OK) == 0) {
    m_runtime_bin = bundled_crun;
    LOG(INFO) << "Using bundled OCI runtime: " << m_runtime_bin;
    m_initialized = true;
    return true;
  }

  // Check if crun exists, fallback to runc
  if (std::system("crun --version > /dev/null 2>&1") == 0) {
    m_runtime_bin = "crun";
  } else if (std::system("runc --version > /dev/null 2>&1") == 0) {
    m_runtime_bin = "runc";
  } else {
    LOG(ERROR) << "Neither crun nor runc found. Using mock runtime for build/tests.";
    m_runtime_bin = "mock_runc";
    // Keep initialization successful for GBS unit-test environment.
  }

  LOG(INFO) << "Using OCI runtime: " << m_runtime_bin;
  m_initialized = true;
  return true;
}

std::string ContainerEngine::ExecuteSkill(const std::string& skill_name, const std::string& arg_str) {
  if (!m_initialized) {
    LOG(ERROR) << "Cannot run skill. Engine not initialized.";
    return "{}";
  }

  if (m_runtime_bin == "mock_runc") {
    LOG(WARNING) << "Mock runtime active. Skip skill execution.";
    return "{}";
  }

  if (!EnsureSkillsContainerRunning()) {
    LOG(ERROR) << "Secure skills container is unavailable.";
    return "{}";
  }

  std::string claw_env = "CLAW_ARGS=" + arg_str;
  std::string skill_path = "/skills/" + skill_name + "/" + skill_name + ".py";

  std::string run_cmd = m_runtime_bin + " exec --env " +
                        EscapeShellArg(claw_env) + " " + m_container_id +
                        " python3 " + EscapeShellArg(skill_path) + " 2>&1";
  LOG(INFO) << "Exec skill in secure container: " << skill_name;

  std::array<char, 256> buffer;
  std::string output;
  std::unique_ptr<FILE, int (*)(FILE*)> pipe(
      popen(run_cmd.c_str(), "r"), pclose);
  if (!pipe) {
    LOG(ERROR) << "popen() failed while executing skill.";
    return "{}";
  }

  while (fgets(buffer.data(), buffer.size(), pipe.get()) != nullptr) {
    output += buffer.data();
  }

  int rc = pclose(pipe.release());
  if (rc != 0) {
    LOG(ERROR) << "Skill command failed: " << rc << ", output: " << output;
    return "{}";
  }

  return output;
}

bool ContainerEngine::EnsureSkillsContainerRunning() {
  if (IsContainerRunning()) {
    return true;
  }

  if (!PrepareSkillsBundle()) {
    return false;
  }

  if (!StartSkillsContainer()) {
    // Auto-restart: force cleanup and try once more
    LOG(WARNING) << "Container start failed. Attempting auto-restart...";
    StopSkillsContainer();
    return StartSkillsContainer();
  }
  return true;
}

bool ContainerEngine::PrepareSkillsBundle() {
  std::string rootfs_dir = m_bundle_dir + "/rootfs";
  std::string marker = m_bundle_dir + "/.extracted";

  std::string prepare_cmd =
      "mkdir -p " + EscapeShellArg(rootfs_dir) + " && " + "if [ ! -f " +
      EscapeShellArg(marker) + " ]; then " + "tar -xzf " +
      EscapeShellArg(m_rootfs_tar) + " -C " + EscapeShellArg(rootfs_dir) +
      " && touch " + EscapeShellArg(marker) + "; fi";

  int ret = std::system(prepare_cmd.c_str());
  if (ret != 0) {
    LOG(ERROR) << "Failed to prepare secure bundle/rootfs. Return: " << ret;
    return false;
  }

  return WriteSkillsConfig();
}

bool ContainerEngine::IsContainerRunning() const {
  std::string check_cmd =
      m_runtime_bin + " state " + m_container_id + " > /dev/null 2>&1";
  return std::system(check_cmd.c_str()) == 0;
}

bool ContainerEngine::StartSkillsContainer() {
  std::string delete_cmd =
      m_runtime_bin + " delete -f " + m_container_id + " > /dev/null 2>&1";
  int delete_ret = std::system(delete_cmd.c_str());
  if (delete_ret != 0) {
    LOG(WARNING) << "Pre-delete secure container returned: " << delete_ret;
  }

  // Workaround for Tizen emulator: disable cgroup manager if using crun to prevent watchdog reboots
  std::string cgroup_arg = "";
  if (m_runtime_bin.find("crun") != std::string::npos) {
    cgroup_arg = " --cgroup-manager=disabled";
  }

  std::string run_cmd = "cd " + EscapeShellArg(m_bundle_dir) + " && " +
                        m_runtime_bin + " run" + cgroup_arg + " -d " + m_container_id +
                        " > /dev/null 2>&1";
  int ret = std::system(run_cmd.c_str());
  if (ret != 0) {
    LOG(ERROR) << "Failed to start secure skills container. Return: " << ret;
    return false;
  }
  return true;
}

void ContainerEngine::StopSkillsContainer() {
  if (!m_initialized || m_runtime_bin == "mock_runc") {
    return;
  }

  std::string stop_cmd =
      m_runtime_bin + " delete -f " + m_container_id + " > /dev/null 2>&1";
  int stop_ret = std::system(stop_cmd.c_str());
  if (stop_ret != 0) {
    LOG(WARNING) << "Delete secure container returned: " << stop_ret;
  }
}

bool ContainerEngine::WriteSkillsConfig() const {
  std::string config_file = m_bundle_dir + "/config.json";
  std::ofstream out_conf(config_file);
  if (!out_conf.is_open()) {
    LOG(ERROR) << "Failed to write secure config.json";
    return false;
  }

  std::string config_json = R"({
  "ociVersion": "1.0.2",
  "process": {
    "terminal": false,
    "user": {"uid": 65534, "gid": 65534},
    "args": ["sh", "-lc", "while true; do sleep 3600; done"],
    "env": [
      "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
    ],
    "cwd": "/",
    "noNewPrivileges": true,
    "capabilities": {
      "bounding": [],
      "effective": [],
      "inheritable": [],
      "permitted": [],
      "ambient": []
    },
    "rlimits": [
      {"type": "RLIMIT_NOFILE", "hard": 256, "soft": 256},
      {"type": "RLIMIT_NPROC", "hard": 64, "soft": 64},
      {"type": "RLIMIT_AS", "hard": 268435456, "soft": 268435456}
    ]
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
      "type": "tmpfs",
      "source": "tmpfs",
      "options": ["nosuid", "strictatime", "mode=755", "size=65536k"]
    },
    {
      "destination": "/skills",
      "type": "bind",
      "source": ")" + m_skills_dir + R"(",
      "options": ["rbind", "ro"]
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
    "seccomp": {
      "defaultAction": "SCMP_ACT_ERRNO",
      "architectures": ["SCMP_ARCH_X86_64", "SCMP_ARCH_X86", "SCMP_ARCH_AARCH64"],
      "syscalls": [{
        "names": [
          "read","write","open","close","stat","fstat","lstat",
          "poll","lseek","mmap","mprotect","munmap","brk",
          "ioctl","access","pipe","select","sched_yield",
          "dup","dup2","nanosleep","getpid","socket","connect",
          "sendto","recvfrom","sendmsg","recvmsg","bind","listen",
          "getsockname","getpeername","getsockopt","setsockopt",
          "clone","fork","vfork","execve","exit","wait4",
          "kill","uname","fcntl","flock","fsync","fdatasync",
          "truncate","ftruncate","getdents","getcwd","chdir",
          "mkdir","rmdir","creat","link","unlink","symlink",
          "readlink","chmod","chown","lchown","umask",
          "gettimeofday","getrlimit","getrusage","sysinfo",
          "times","getuid","getgid","setuid","setgid",
          "geteuid","getegid","getppid","getpgrp","setsid",
          "getgroups","setgroups","sigaltstack","madvise",
          "shmget","shmat","shmctl","shmdt",
          "clock_gettime","clock_getres","clock_nanosleep",
          "exit_group","epoll_wait","epoll_ctl","tgkill",
          "openat","mkdirat","fchownat","fstatat",
          "unlinkat","renameat","linkat","symlinkat",
          "readlinkat","fchmodat","faccessat","futex",
          "set_robust_list","get_robust_list",
          "epoll_create1","pipe2","dup3","accept4",
          "prlimit64","getrandom","memfd_create",
          "statx","clone3","close_range","rseq",
          "newfstatat"
        ],
        "action": "SCMP_ACT_ALLOW"
      }]
    },
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
})";
  out_conf << config_json;
  out_conf.close();
  return true;
}

std::string ContainerEngine::BuildPaths(const std::string& leaf) const {
  if (leaf.empty()) {
    return m_app_data_dir;
  }
  return m_app_data_dir + "/" + leaf;
}

std::string ContainerEngine::EscapeShellArg(const std::string& input) const {
  std::string output = "'";
  for (char c : input) {
    if (c == '\'') {
      output += "'\\''";
    } else {
      output += c;
    }
  }
  output += "'";
  return output;
}

} // namespace tizenclaw
