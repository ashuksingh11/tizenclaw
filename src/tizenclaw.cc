#include "tizenclaw.hh"

#include <iostream>
#include <string>
#include <csignal>
#include <exception>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>
#include <cstring>
#include <vector>

TizenClawDaemon* g_daemon = nullptr;

void signal_handler(int sig) {
    dlog_print(DLOG_INFO, LOG_TAG, "Caught signal %d", sig);
    if (g_daemon) {
        g_daemon->Quit();
    }
}

TizenClawDaemon::TizenClawDaemon(int argc, char** argv)
    : argc_(argc), argv_(argv) {
    tizen_core_init();
    tizen_core_task_create("main", false, &task_);
}

TizenClawDaemon::~TizenClawDaemon() {
    if (task_) {
        tizen_core_task_destroy(task_);
        task_ = nullptr;
    }
    tizen_core_shutdown();
}

int TizenClawDaemon::Run() {
    dlog_print(DLOG_INFO, LOG_TAG, "TizenClaw Daemon Run");
    OnCreate();
    
    // Set up signal handling
    std::signal(SIGINT, signal_handler);
    std::signal(SIGTERM, signal_handler);

    int ret = tizen_core_task_run(task_);
    OnDestroy();
    return ret;
}

void TizenClawDaemon::Quit() {
    dlog_print(DLOG_INFO, LOG_TAG, "TizenClaw Daemon Quit");
    if (task_) {
        tizen_core_task_quit(task_);
    }
}

void TizenClawDaemon::OnCreate() {
    dlog_print(DLOG_INFO, LOG_TAG, "TizenClaw Daemon OnCreate");
    agent_ = new AgentCore();
    if (!agent_->Initialize()) {
        dlog_print(DLOG_ERROR, LOG_TAG, "Failed to initialize AgentCore");
    }

    // TODO: Initialize LXC Container Engine
    // TODO: Start MCP Server connection
    
    ipc_running_ = true;
    ipc_thread_ = std::thread(&TizenClawDaemon::IpcServerLoop, this);
}

void TizenClawDaemon::OnDestroy() {
    dlog_print(DLOG_INFO, LOG_TAG, "TizenClaw Daemon OnDestroy");
    
    ipc_running_ = false;
    if (ipc_socket_ != -1) {
        shutdown(ipc_socket_, SHUT_RDWR);
        close(ipc_socket_);
        ipc_socket_ = -1;
    }
    if (ipc_thread_.joinable()) {
        ipc_thread_.join();
    }

    if (agent_) {
        agent_->Shutdown();
        delete agent_;
        agent_ = nullptr;
    }
    
    // TODO: Cleanup LXC processes and MCP sockets here
}

void TizenClawDaemon::IpcServerLoop() {
    dlog_print(DLOG_INFO, LOG_TAG, "IPC Server thread starting...");
    
    int sock = socket(AF_UNIX, SOCK_STREAM, 0);
    if (sock < 0) {
        dlog_print(DLOG_ERROR, LOG_TAG, "Failed to create IPC socket: %s", strerror(errno));
        return;
    }
    ipc_socket_ = sock;
    
    struct sockaddr_un addr;
    std::memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    
    // Abstract namespace socket: "\0tizenclaw.ipc" = 14 bytes
    // addr.sun_path[0] is already '\0' from memset (abstract namespace indicator)
    // Copy "tizenclaw.ipc" starting at sun_path[1]
    const char kSocketName[] = "tizenclaw.ipc";
    constexpr size_t kNameLen = 1 + sizeof(kSocketName) - 1;  // '\0' + "tizenclaw.ipc" = 14
    std::memcpy(addr.sun_path + 1, kSocketName, sizeof(kSocketName) - 1);
    
    socklen_t addr_len = offsetof(struct sockaddr_un, sun_path) + kNameLen;
    
    if (bind(ipc_socket_, (struct sockaddr*)&addr, addr_len) < 0) {
        dlog_print(DLOG_ERROR, LOG_TAG, "Failed to bind IPC socket: %s", strerror(errno));
        close(ipc_socket_);
        ipc_socket_ = -1;
        return;
    }
    
    if (listen(ipc_socket_, 5) < 0) {
        dlog_print(DLOG_ERROR, LOG_TAG, "Failed to listen on IPC socket: %s", strerror(errno));
        close(ipc_socket_);
        ipc_socket_ = -1;
        return;
    }
    
    dlog_print(DLOG_INFO, LOG_TAG, "IPC Server listening on abstract socket \\0tizenclaw.ipc (addr_len=%d)", addr_len);
    
    while (ipc_running_) {
        int client_sock = accept(ipc_socket_, nullptr, nullptr);
        if (client_sock < 0) {
            if (ipc_running_) {
                dlog_print(DLOG_WARN, LOG_TAG, "accept() failed: %s", strerror(errno));
            }
            continue;
        }
        
        dlog_print(DLOG_INFO, LOG_TAG, "IPC client connected");
        
        std::vector<char> buffer(4096);
        std::string prompt;
        ssize_t bytes_read;
        while ((bytes_read = ::read(client_sock, buffer.data(), buffer.size())) > 0) {
            prompt.append(buffer.data(), bytes_read);
        }
        
        close(client_sock);
        
        if (!prompt.empty() && agent_) {
            dlog_print(DLOG_INFO, LOG_TAG, "Received IPC prompt (%zu bytes): %s",
                        prompt.size(), prompt.c_str());
            agent_->ProcessPrompt(prompt);
        }
    }
    
    dlog_print(DLOG_INFO, LOG_TAG, "IPC Server thread exiting...");
}

int main(int argc, char *argv[]) {
    dlog_print(DLOG_INFO, LOG_TAG, "TizenClaw Service starting...");
    try {
        TizenClawDaemon daemon(argc, argv);
        g_daemon = &daemon;
        return daemon.Run();
    } catch (const std::exception& e) {
        dlog_print(DLOG_ERROR, LOG_TAG, "Exception: %s", e.what());
        return -1;
    } catch (...) {
        dlog_print(DLOG_ERROR, LOG_TAG, "Unknown exception");
        return -1;
    }
}
