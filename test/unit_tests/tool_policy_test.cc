#include <gtest/gtest.h>
#include <fstream>
#include <cstdlib>
#include <unistd.h>
#include "tool_policy.hh"

using namespace tizenclaw;


class ToolPolicyTest : public ::testing::Test {
protected:
    void SetUp() override {
        policy = new ToolPolicy();
    }

    void TearDown() override {
        delete policy;
        unlink("test_tool_policy.json");
    }

    ToolPolicy* policy;
};

TEST_F(ToolPolicyTest, DefaultPolicyAllowsAll) {
    // Without loading config, all tools allowed
    nlohmann::json args = {{"app_id", "test"}};
    std::string violation =
        policy->CheckPolicy(
            "session1", "launch_app", args);
    EXPECT_TRUE(violation.empty());
}

TEST_F(ToolPolicyTest, LoadConfigFromFile) {
    std::ofstream f("test_tool_policy.json");
    f << R"({
      "max_repeat_count": 2,
      "blocked_skills": ["dangerous_tool"],
      "risk_overrides": {
        "launch_app": "high"
      }
    })" << std::endl;
    f.close();

    EXPECT_TRUE(policy->LoadConfig(
        "test_tool_policy.json"));
}

TEST_F(ToolPolicyTest, BlockedSkillRejected) {
    std::ofstream f("test_tool_policy.json");
    f << R"({
      "blocked_skills": ["blocked_tool"]
    })" << std::endl;
    f.close();

    policy->LoadConfig("test_tool_policy.json");

    std::string violation =
        policy->CheckPolicy(
            "s1", "blocked_tool", {});
    EXPECT_FALSE(violation.empty());
    EXPECT_NE(violation.find("blocked"),
              std::string::npos);
}

TEST_F(ToolPolicyTest, LoopDetectionBlocks) {
    std::ofstream f("test_tool_policy.json");
    f << R"({"max_repeat_count": 2})"
      << std::endl;
    f.close();

    policy->LoadConfig("test_tool_policy.json");

    nlohmann::json args = {{"app_id", "test"}};

    // First two calls should pass
    EXPECT_TRUE(policy->CheckPolicy(
        "s1", "launch_app", args).empty());
    EXPECT_TRUE(policy->CheckPolicy(
        "s1", "launch_app", args).empty());

    // Third call should be blocked (loop)
    std::string violation =
        policy->CheckPolicy(
            "s1", "launch_app", args);
    EXPECT_FALSE(violation.empty());
    EXPECT_NE(violation.find("loop"),
              std::string::npos);
}

TEST_F(ToolPolicyTest,
       DifferentArgsNotBlocked) {
    std::ofstream f("test_tool_policy.json");
    f << R"({"max_repeat_count": 1})"
      << std::endl;
    f.close();

    policy->LoadConfig("test_tool_policy.json");

    nlohmann::json args1 = {{"app_id", "app1"}};
    nlohmann::json args2 = {{"app_id", "app2"}};

    EXPECT_TRUE(policy->CheckPolicy(
        "s1", "launch_app", args1).empty());
    // Different args = different hash
    EXPECT_TRUE(policy->CheckPolicy(
        "s1", "launch_app", args2).empty());
}

TEST_F(ToolPolicyTest,
       DifferentSessionsIndependent) {
    std::ofstream f("test_tool_policy.json");
    f << R"({"max_repeat_count": 1})"
      << std::endl;
    f.close();

    policy->LoadConfig("test_tool_policy.json");

    nlohmann::json args = {{"app_id", "test"}};

    // Session 1: first call
    EXPECT_TRUE(policy->CheckPolicy(
        "s1", "launch_app", args).empty());

    // Session 2: independent tracking
    EXPECT_TRUE(policy->CheckPolicy(
        "s2", "launch_app", args).empty());
}

TEST_F(ToolPolicyTest, ResetSessionClears) {
    std::ofstream f("test_tool_policy.json");
    f << R"({"max_repeat_count": 1})"
      << std::endl;
    f.close();

    policy->LoadConfig("test_tool_policy.json");

    nlohmann::json args = {{"app_id", "test"}};

    // Use up the limit
    EXPECT_TRUE(policy->CheckPolicy(
        "s1", "launch_app", args).empty());
    EXPECT_FALSE(policy->CheckPolicy(
        "s1", "launch_app", args).empty());

    // Reset session tracking
    policy->ResetSession("s1");

    // Should be allowed again
    EXPECT_TRUE(policy->CheckPolicy(
        "s1", "launch_app", args).empty());
}

TEST_F(ToolPolicyTest,
       ManifestRiskLevelLoaded) {
    nlohmann::json manifest = {
        {"name", "launch_app"},
        {"risk_level", "high"}
    };

    policy->LoadManifestRiskLevel(
        "launch_app", manifest);

    EXPECT_EQ(policy->GetRiskLevel("launch_app"),
              RiskLevel::kHigh);
}

TEST_F(ToolPolicyTest,
       DefaultRiskLevelIsNormal) {
    EXPECT_EQ(
        policy->GetRiskLevel("unknown_tool"),
        RiskLevel::kNormal);
}

TEST_F(ToolPolicyTest,
       RiskLevelToStringWorks) {
    EXPECT_EQ(ToolPolicy::RiskLevelToString(
        RiskLevel::kLow), "low");
    EXPECT_EQ(ToolPolicy::RiskLevelToString(
        RiskLevel::kNormal), "normal");
    EXPECT_EQ(ToolPolicy::RiskLevelToString(
        RiskLevel::kHigh), "high");
}

TEST_F(ToolPolicyTest,
       MissingConfigUsesDefaults) {
    EXPECT_TRUE(policy->LoadConfig(
        "/nonexistent/path.json"));
}
