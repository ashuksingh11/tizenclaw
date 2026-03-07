// Copyright 2026 TizenClaw Authors
// Supervisor Agent Pattern — role-based multi-agent
// orchestration with hierarchical delegation
#ifndef TIZENCLAW_CORE_AGENT_ROLE_H_
#define TIZENCLAW_CORE_AGENT_ROLE_H_

#include <map>
#include <mutex>
#include <string>
#include <vector>

#include <json.hpp>

namespace tizenclaw {

class AgentCore;  // forward declaration

// Agent role definition loaded from
// agent_roles.json
struct AgentRole {
  std::string name;
  std::string system_prompt;
  std::vector<std::string> allowed_tools;
  int max_iterations = 10;
};

// Result of a single delegation
struct DelegationResult {
  std::string role_name;
  std::string session_id;
  std::string sub_task;
  std::string result;
  bool success = false;
};

// Supervisor engine for multi-agent
// orchestration
class SupervisorEngine {
 public:
  explicit SupervisorEngine(AgentCore* agent);

  // Load role definitions from JSON config
  bool LoadRoles(const std::string& config_path);

  // Run supervisor loop:
  // decompose → delegate → collect → validate
  std::string RunSupervisor(
      const std::string& goal,
      const std::string& strategy,
      const std::string& session_id);

  // List configured roles
  nlohmann::json ListRoles() const;

  // Get role by name (nullptr if not found)
  const AgentRole* GetRole(
      const std::string& name) const;

  // Get all role names
  std::vector<std::string> GetRoleNames() const;

 private:
  // Decompose goal into (role, sub_task) pairs
  // via LLM
  std::vector<std::pair<std::string, std::string>>
  DecomposeGoal(
      const std::string& goal,
      const std::string& session_id);

  // Delegate sub-task to a role agent session
  DelegationResult DelegateToRole(
      const AgentRole& role,
      const std::string& sub_task,
      const std::string& parent_session);

  // Validate and aggregate results via LLM
  std::string ValidateResults(
      const std::string& goal,
      const std::vector<DelegationResult>& results,
      const std::string& session_id);

  AgentCore* agent_;
  std::map<std::string, AgentRole> roles_;
  mutable std::mutex roles_mutex_;

  // Session prefix for role agents
  static constexpr const char* kRolePrefix =
      "role_";
};

}  // namespace tizenclaw

#endif  // TIZENCLAW_CORE_AGENT_ROLE_H_
