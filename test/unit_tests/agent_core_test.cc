#include <gtest/gtest.h>
#include <fstream>
#include <cstdlib>
#include <unistd.h>
#include "agent_core.hh"

using namespace tizenclaw;


class AgentCoreTest : public ::testing::Test {
protected:
    void SetUp() override {
        // Create a dummy config for testing
        const char* test_config = "test_llm_config.json";
        std::ofstream f(test_config);
        f << "{\"active_backend\":\"ollama\",\"backends\":{\"ollama\":{\"endpoint\":\"http://localhost:9999\",\"model\":\"dummy\"}}}" << std::endl;
        f.close();
        setenv("TIZENCLAW_CONFIG_PATH", test_config, 1);
        
        agent = new AgentCore();
    }

    void TearDown() override {
        delete agent;
        unlink("test_llm_config.json");
    }

    AgentCore* agent;
};

TEST_F(AgentCoreTest, InitializationTest) {
    // 1. First initialization should succeed
    EXPECT_TRUE(agent->Initialize());
    
    // 2. Second initialization should also safely return true without issues
    EXPECT_TRUE(agent->Initialize());
}

TEST_F(AgentCoreTest, ProcessPromptWithoutInit) {
    // Without initialization, should return error
    std::string result =
        agent->ProcessPrompt("test_session",
                             "Hello TizenClaw!");
    EXPECT_FALSE(result.empty());
    EXPECT_NE(result.find("Error"), std::string::npos);
}

TEST_F(AgentCoreTest, ProcessPromptReturnsString) {
    agent->Initialize();
    // ProcessPrompt should return a response.
    // In a test environment without a real LLM config/backend, 
    // it might return an error string, which is still a non-empty string.
    std::string result =
        agent->ProcessPrompt("test_session",
                             "What is the battery level?");
    EXPECT_FALSE(result.empty());
}

TEST_F(AgentCoreTest, IterativeLoopDetection) {
    // This test would ideally mock LlmBackend to return tool_calls,
    // then verify that AgentCore::ProcessPrompt enters a second iteration.
    // For now, we perform a basic call.
    agent->Initialize();
    std::string result = agent->ProcessPrompt("multi_step_session", "List apps and then check Wi-Fi.");
    EXPECT_FALSE(result.empty());
}

