#include <gtest/gtest.h>
#include "agent_core.h"

class AgentCoreTest : public ::testing::Test {
protected:
    void SetUp() override {
        agent = new AgentCore();
    }

    void TearDown() override {
        delete agent;
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
    // Test that the agent handles prompt processing safely without initialization
    // Since we redirect dlog_print to printf in main.cc, this won't crash and will just print the error.
    ASSERT_NO_THROW({
        agent->ProcessPrompt("Hello TizenClaw!");
    });
}
