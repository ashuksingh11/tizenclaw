#include "container_engine.hh"

#include "../common/logging.hh"
#include <array>
#include <cerrno>
#include <cstdio>
#include <cstdlib>
#include <fstream>
#include <json.hpp>
#include <memory>
#include <string>
#include <unistd.h>
#include <sys/wait.h>

namespace tizenclaw {

// Custom command runner using fork/exec with /bin/bash.
// We cannot use popen() because it invokes /bin/sh, which in the standard
// container is busybox linked against musl libc — but /lib is bind-mounted
// from the host (glibc), making busybox non-functional.
static std::pair<std::string, int> RunCommand(const std::string& cmd) {
  int pipefd[2];
  if (pipe(pipefd) == -1) {
    return {"", -1};
  }

  pid_t pid = fork();
  if (pid == -1) {
    close(pipefd[0]);
    close(pipefd[1]);
    return {"", -1};
  }

  if (pid == 0) {
    // Child: redirect stdout+stderr to pipe write end
    close(pipefd[0]);
    dup2(pipefd[1], STDOUT_FILENO);
    dup2(pipefd[1], STDERR_FILENO);
    close(pipefd[1]);
    // Use /usr/bin/bash (from host bind-mount, glibc-linked) instead of
    // /bin/bash (from rootfs, musl-linked and broken by host /lib mount).
    execl("/usr/bin/bash", "bash", "-c", cmd.c_str(), nullptr);
    _exit(127);
  }

  // Parent: read from pipe
  close(pipefd[1]);
  std::string output;
  char buffer[256];
  ssize_t n;
  while ((n = read(pipefd[0], buffer, sizeof(buffer) - 1)) > 0) {
    buffer[n] = '\0';
    output += buffer;
  }
  close(pipefd[0]);

  int status = 0;
  waitpid(pid, &status, 0);
  int rc = WIFEXITED(status) ? WEXITSTATUS(status) : -1;
  return {output, rc};
}

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

  std::string claw_env = "CLAW_ARGS=" + arg_str;
  std::string skill_path = "/skills/" + skill_name + "/" + skill_name + ".py";
  std::string run_cmd;

  // Try OCI container first, fallback to chroot
  if (EnsureSkillsContainerRunning()) {
    run_cmd = m_runtime_bin + " exec --env " +
              EscapeShellArg(claw_env) + " " + m_container_id +
              " python3 " + EscapeShellArg(skill_path) + " 2>&1";
    LOG(INFO) << "Exec skill in OCI container: " << skill_name;
  } else {
    // Fallback: run skill directly on host when OCI container is unavailable.
    // This gives up container isolation but avoids ABI mismatches between
    // host Tizen CAPI libraries and the container rootfs's glibc.
    std::string host_skill_path = m_skills_dir + "/" + skill_name + "/" +
                                  skill_name + ".py";
    if (access(host_skill_path.c_str(), R_OK) != 0) {
      LOG(ERROR) << "Skill script not found: " << host_skill_path;
      nlohmann::json err;
      err["error"] = "Skill script not found: " + host_skill_path;
      return err.dump();
    }

    LOG(WARNING) << "OCI container unavailable. Running skill directly on host: "
                 << skill_name;
    // RunCommand() invokes /bin/bash -c internally, so run_cmd is just the
    // shell command to execute.
    run_cmd = "CLAW_ARGS=" + EscapeShellArg(arg_str) +
              " /usr/bin/python3 " + EscapeShellArg(host_skill_path) +
              " 2>&1";
  }

  LOG(DEBUG) << "Running command: " << run_cmd;
  auto [output, rc] = RunCommand(run_cmd);
  if (rc == -1 && output.empty()) {
    LOG(ERROR) << "fork/exec failed for skill command.";
    return "{}";
  }

  if (rc != 0) {
    LOG(ERROR) << "Skill command failed: " << rc << ", output: " << output;
    // Return structured error so LLM can generate a meaningful response
    nlohmann::json err;
    err["error"] = "Skill execution failed with exit code " + std::to_string(rc);
    err["details"] = output.length() > 500 ? output.substr(0, 500) : output;
    return err.dump();
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
      "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
      "LD_LIBRARY_PATH=/tizen_libs"
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
      "type": "bind",
      "source": "/dev",
      "options": ["rbind", "ro"]
    },
    {
      "destination": "/skills",
      "type": "bind",
      "source": ")" + m_skills_dir + R"(",
      "options": ["rbind", "ro"]
    },
    {
      "destination": "/tizen_libs",
      "type": "bind",
      "source": "/usr/lib",
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
