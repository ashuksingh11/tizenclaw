/*
 * Copyright (c) 2026 Samsung Electronics Co., Ltd
 * All Rights Reserved
 *
 * Licensed under the Apache License, Version 2.0
 */

#ifndef TIZENCLAW_CORE_TOOL_POLICY_H_
#define TIZENCLAW_CORE_TOOL_POLICY_H_

#include <map>
#include <mutex>
#include <set>
#include <string>
#include <vector>
#include <json.hpp>

namespace tizenclaw {

enum class RiskLevel {
  kLow,     // Read-only (get_battery_info, etc.)
  kNormal,  // Default
  kHigh,    // Side-effect (launch_app, etc.)
};

struct ToolPolicyConfig {
  // Per-skill risk level overrides
  std::map<std::string, RiskLevel> risk_levels;
  // Max repeated calls (same skill + same args)
  int max_repeat_count = 3;
  // Skills blocked entirely
  std::set<std::string> blocked_skills;
  // Max agentic loop iterations
  int max_iterations = 5;
};

class ToolPolicy {
public:
  ToolPolicy();

  // Load policy config from JSON file
  // Returns true if loaded (or defaults used)
  bool LoadConfig(const std::string& config_path);

  // Load risk_level from skill manifest
  void LoadManifestRiskLevel(
      const std::string& skill_name,
      const nlohmann::json& manifest);

  // Check if a tool call is allowed.
  // Returns empty string if allowed,
  // violation reason string if blocked.
  std::string CheckPolicy(
      const std::string& session_id,
      const std::string& skill_name,
      const nlohmann::json& args);

  // Track iteration outputs for idle detection.
  // Returns true if idle (no progress).
  bool CheckIdleProgress(
      const std::string& session_id,
      const std::string& iteration_output);

  // Get max iterations for agentic loop
  int GetMaxIterations() const;

  // Reset per-session call tracking
  void ResetSession(
      const std::string& session_id);

  // Reset idle tracking for a session
  void ResetIdleTracking(
      const std::string& session_id);

  // Get risk level for a skill
  RiskLevel GetRiskLevel(
      const std::string& skill_name) const;

  // Convert RiskLevel to string
  static std::string RiskLevelToString(
      RiskLevel level);

private:
  // Generate hash key for loop detection
  std::string HashCall(
      const std::string& name,
      const nlohmann::json& args) const;

  // Parse risk level string to enum
  static RiskLevel ParseRiskLevel(
      const std::string& str);

  ToolPolicyConfig config_;

  // Track repeated calls per session:
  // session_id -> {call_hash -> count}
  std::map<std::string,
      std::map<std::string, int>> call_history_;

  // Track iteration outputs for idle detection
  // session_id -> recent iteration signatures
  std::map<std::string,
      std::vector<std::string>> idle_history_;
  static constexpr int kIdleWindowSize = 3;

  std::mutex mutex_;
};

}  // namespace tizenclaw

#endif  // TIZENCLAW_CORE_TOOL_POLICY_H_
