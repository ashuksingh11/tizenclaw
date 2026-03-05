#include "container_engine.hh"

#include <dlog.h>
#include <array>
#include <cstdio>
#include <cstdlib>
#include <fstream>
#include <memory>
#include <string>
#include <unistd.h>

#ifdef  LOG_TAG
#undef  LOG_TAG
#endif
#define LOG_TAG "TizenClaw_Container"

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

  dlog_print(DLOG_INFO, LOG_TAG, "ContainerEngine Initializing...");

  const char* bundled_crun = "/usr/libexec/tizenclaw/crun";
  if (access(bundled_crun, X_OK) == 0) {
    m_runtime_bin = bundled_crun;
    dlog_print(DLOG_INFO, LOG_TAG, "Using bundled OCI runtime: %s",
               m_runtime_bin.c_str());
    m_initialized = true;
    return true;
  }

  // Check if crun exists, fallback to runc
  if (std::system("crun --version > /dev/null 2>&1") == 0) {
    m_runtime_bin = "crun";
  } else if (std::system("runc --version > /dev/null 2>&1") == 0) {
    m_runtime_bin = "runc";
  } else {
    dlog_print(
        DLOG_ERROR, LOG_TAG,
        "Neither crun nor runc found. Using mock runtime for build/tests.");
    m_runtime_bin = "mock_runc";
    // Keep initialization successful for GBS unit-test environment.
  }

  dlog_print(DLOG_INFO, LOG_TAG, "Using OCI runtime: %s", m_runtime_bin.c_str());
  m_initialized = true;
  return true;
}

std::string ContainerEngine::ExecuteSkill(const std::string& skill_name, const std::string& arg_str) {
  if (!m_initialized) {
    dlog_print(DLOG_ERROR, LOG_TAG, "Cannot run skill. Engine not initialized.");
    return "{}";
  }

  if (m_runtime_bin == "mock_runc") {
    dlog_print(DLOG_WARN, LOG_TAG, "Mock runtime active. Skip skill execution.");
    return "{}";
  }

  if (!EnsureSkillsContainerRunning()) {
    dlog_print(DLOG_ERROR, LOG_TAG, "Secure skills container is unavailable.");
    return "{}";
  }

  std::string claw_env = "CLAW_ARGS=" + arg_str;
  std::string skill_path = "/skills/" + skill_name + "/" + skill_name + ".py";

  std::string run_cmd = m_runtime_bin + " exec --env " +
                        EscapeShellArg(claw_env) + " " + m_container_id +
                        " python3 " + EscapeShellArg(skill_path) + " 2>&1";
  dlog_print(DLOG_INFO, LOG_TAG, "Exec skill in secure container: %s",
             skill_name.c_str());

  std::array<char, 256> buffer;
  std::string output;
  std::unique_ptr<FILE, int (*)(FILE*)> pipe(
      popen(run_cmd.c_str(), "r"), pclose);
  if (!pipe) {
    dlog_print(DLOG_ERROR, LOG_TAG, "popen() failed while executing skill.");
    return "{}";
  }

  while (fgets(buffer.data(), buffer.size(), pipe.get()) != nullptr) {
    output += buffer.data();
  }

  int rc = pclose(pipe.release());
  if (rc != 0) {
    dlog_print(DLOG_ERROR, LOG_TAG, "Skill command failed: %d, output: %s", rc,
               output.c_str());
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
  return StartSkillsContainer();
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
    dlog_print(DLOG_ERROR, LOG_TAG,
               "Failed to prepare secure bundle/rootfs. Return: %d", ret);
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
    dlog_print(DLOG_WARN, LOG_TAG,
               "Pre-delete secure container returned: %d", delete_ret);
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
    dlog_print(DLOG_ERROR, LOG_TAG,
               "Failed to start secure skills container. Return: %d", ret);
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
    dlog_print(DLOG_WARN, LOG_TAG,
               "Delete secure container returned: %d", stop_ret);
  }
}

bool ContainerEngine::WriteSkillsConfig() const {
  std::string config_file = m_bundle_dir + "/config.json";
  std::ofstream out_conf(config_file);
  if (!out_conf.is_open()) {
    dlog_print(DLOG_ERROR, LOG_TAG, "Failed to write secure config.json");
    return false;
  }

  std::string config_json = R"({
  "ociVersion": "1.0.2",
  "process": {
    "terminal": false,
    "user": {"uid": 0, "gid": 0},
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
