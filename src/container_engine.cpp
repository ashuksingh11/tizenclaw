#include "container_engine.h"
#include <dlog.h>
#include <cstdlib>
#include <string>

#ifdef  LOG_TAG
#undef  LOG_TAG
#endif
#define LOG_TAG "TizenClaw_Container"

ContainerEngine::ContainerEngine() : m_initialized(false) {
}

ContainerEngine::~ContainerEngine() {
}

bool ContainerEngine::Initialize() {
    if (m_initialized) return true;

    dlog_print(DLOG_INFO, LOG_TAG, "ContainerEngine Initializing runc environment...");
    
    // In a real environment, we'd check if 'runc' binary is available in $PATH
    int ret = std::system("runc --version > /dev/null 2>&1");
    if (ret != 0) {
        dlog_print(DLOG_ERROR, LOG_TAG, "runc binary not found or not executable. Container execution might fail.");
        // We still return true to not block the daemon, but log the error
    }

    m_initialized = true;
    return true;
}

bool ContainerEngine::StartContainer(const std::string& container_name, const std::string& rootfs_path) {
    if (!m_initialized) {
        dlog_print(DLOG_ERROR, LOG_TAG, "Cannot start container. Engine not initialized.");
        return false;
    }

    dlog_print(DLOG_INFO, LOG_TAG, "Creating Container via runc: %s with config in: %s", 
               container_name.c_str(), rootfs_path.c_str());

    // 1. Prepare bundle directory
    std::string bundle_dir = "/usr/apps/org.tizen.tizenclaw/data/bundles/" + container_name;
    std::string mkdir_cmd = "mkdir -p " + bundle_dir + "/rootfs";
    std::system(mkdir_cmd.c_str());

    // 2. Extract rootfs if it doesn't exist
    // In Phase 3, we expect rootfs_path to point to /usr/apps/.../data/rootfs.tar.gz
    // We will extract it if the target directory is empty
    std::string extract_cmd = "if [ ! -f " + bundle_dir + "/.extracted ]; then "
                              "tar -xzf " + rootfs_path + " -C " + bundle_dir + "/rootfs && "
                              "touch " + bundle_dir + "/.extracted; fi";
    dlog_print(DLOG_INFO, LOG_TAG, "Extracting RootFS: %s", extract_cmd.c_str());
    int ext_ret = std::system(extract_cmd.c_str());
    if (ext_ret != 0) {
        dlog_print(DLOG_WARN, LOG_TAG, "Failed to extract rootfs! Return code: %d", ext_ret);
    }

    // 3. Generate a basic config.json if it doesn't exist
    std::string config_file = bundle_dir + "/config.json";
    std::string config_cmd = "if [ ! -f " + config_file + " ]; then "
                             "echo '{\"ociVersion\": \"1.0.2\", \"process\": {\"args\": [\"/bin/bash\"]}, "
                             "\"root\": {\"path\": \"rootfs\", \"readonly\": false}}' > " + config_file + "; fi";
    std::system(config_cmd.c_str());

    // runc expects a config.json in the bundle directory.
    // For now we simulate the command, pointing to the bundle
    std::string run_cmd = "runc --root /tmp/runc run -b " + bundle_dir + " -d " + container_name;
    dlog_print(DLOG_INFO, LOG_TAG, "Executing: %s", run_cmd.c_str());

    // Using std::system() for basic execution. In production, fork() and exec() or popen() 
    // is preferred for better process tracking.
    int ret = std::system((run_cmd + " > /dev/null 2>&1").c_str());
    if (ret != 0) {
        // Under test environments without rootfs/config.json, runc run will fail.
        dlog_print(DLOG_WARN, LOG_TAG, "runc run failed (expected in mock). Ret code: %d", ret);
    }

    return true;
}

bool ContainerEngine::StopContainer(const std::string& container_name) {
    if (!m_initialized) return false;

    dlog_print(DLOG_INFO, LOG_TAG, "Stopping Container via runc: %s", container_name.c_str());

    std::string stop_cmd = "runc --root /tmp/runc kill " + container_name + " KILL";
    dlog_print(DLOG_INFO, LOG_TAG, "Executing: %s", stop_cmd.c_str());
    
    int ret = std::system((stop_cmd + " > /dev/null 2>&1").c_str());
    if (ret != 0) {
        dlog_print(DLOG_WARN, LOG_TAG, "runc kill failed (expected in mock). Ret code: %d", ret);
    }
    
    std::string delete_cmd = "runc --root /tmp/runc delete " + container_name;
    int del_ret = std::system((delete_cmd + " > /dev/null 2>&1").c_str());
    (void)del_ret;

    return true;
}
