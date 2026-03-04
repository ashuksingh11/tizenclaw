#include "agent_core.h"
#include <dlog.h>

#ifdef  LOG_TAG
#undef  LOG_TAG
#endif
#define LOG_TAG "TizenClaw_AgentCore"

AgentCore::AgentCore() : m_container(new ContainerEngine()), m_initialized(false) {
    // Constructor
}

AgentCore::~AgentCore() {
    Shutdown();
}

bool AgentCore::Initialize() {
    if (m_initialized) return true;

    dlog_print(DLOG_INFO, LOG_TAG, "AgentCore Initializing...");
    
    if (!m_container->Initialize()) {
        dlog_print(DLOG_ERROR, LOG_TAG, "Failed to initialize LXC Container Engine");
        return false;
    }

    // TODO: Prepare local context for LLM

    m_initialized = true;
    return true;
}

void AgentCore::Shutdown() {
    if (!m_initialized) return;

    dlog_print(DLOG_INFO, LOG_TAG, "AgentCore Shutting down...");
    
    m_container.reset();
    
    m_initialized = false;
}

void AgentCore::ProcessPrompt(const std::string& prompt) {
    if (!m_initialized) {
        dlog_print(DLOG_ERROR, LOG_TAG, "Cannot process prompt. AgentCore not initialized.");
        return;
    }

    dlog_print(DLOG_INFO, LOG_TAG, "AgentCore received prompt: %s", prompt.c_str());

    // TODO: Send prompt to LLM (via MCP or local inference)
    // TODO: Parse the plan
    
    // Launching the predefined container environment for Skills execution
    m_container->StartContainer("tizenclaw_skill_vm", "/usr/apps/org.tizen.tizenclaw/data/rootfs.tar.gz");
}
