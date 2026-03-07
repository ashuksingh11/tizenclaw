// Copyright 2026 TizenClaw Authors
// A2A (Agent-to-Agent) Protocol handler
// implementing Google A2A specification
#ifndef TIZENCLAW_CHANNEL_A2A_HANDLER_H_
#define TIZENCLAW_CHANNEL_A2A_HANDLER_H_

#include <map>
#include <mutex>
#include <string>
#include <vector>

#include <json.hpp>

namespace tizenclaw {

class AgentCore;  // forward declaration

// A2A task status lifecycle
enum class A2ATaskStatus {
  kSubmitted,
  kWorking,
  kInputRequired,
  kCompleted,
  kFailed,
  kCancelled,
};

// A2A task representation
struct A2ATask {
  std::string id;
  A2ATaskStatus status =
      A2ATaskStatus::kSubmitted;
  std::string session_id;
  nlohmann::json message;
  nlohmann::json artifacts;
  std::string created_at;
  std::string updated_at;
};

// A2A protocol handler
class A2AHandler {
 public:
  explicit A2AHandler(AgentCore* agent);

  // Agent Card (/.well-known/agent.json)
  nlohmann::json GetAgentCard() const;

  // JSON-RPC 2.0 method dispatch
  nlohmann::json HandleJsonRpc(
      const nlohmann::json& request);

  // Validate bearer token
  bool ValidateBearerToken(
      const std::string& token) const;

  // Load A2A config (bearer tokens etc.)
  bool LoadConfig(
      const std::string& config_path);

 private:
  // JSON-RPC methods
  nlohmann::json TaskSend(
      const nlohmann::json& params);
  nlohmann::json TaskGet(
      const nlohmann::json& params);
  nlohmann::json TaskCancel(
      const nlohmann::json& params);

  // Helpers
  std::string GenerateTaskId() const;
  std::string GetTimestamp() const;
  std::string TaskStatusToString(
      A2ATaskStatus status) const;

  // JSON-RPC error helpers
  static nlohmann::json JsonRpcError(
      int code,
      const std::string& message,
      const nlohmann::json& id);
  static nlohmann::json JsonRpcResult(
      const nlohmann::json& result,
      const nlohmann::json& id);

  AgentCore* agent_;
  std::map<std::string, A2ATask> tasks_;
  mutable std::mutex tasks_mutex_;

  // Config
  std::vector<std::string> bearer_tokens_;
  std::string agent_name_;
  std::string agent_description_;
  std::string agent_url_;
};

}  // namespace tizenclaw

#endif  // TIZENCLAW_CHANNEL_A2A_HANDLER_H_
