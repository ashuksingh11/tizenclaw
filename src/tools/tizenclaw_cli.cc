/*
 * Copyright (c) 2026 Samsung Electronics Co., Ltd All Rights Reserved
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
/**
 * tizenclaw-cli: CLI tool for testing TizenClaw daemon
 * utilizing the libtizenclaw CAPI.
 *
 * Usage:
 *   tizenclaw-cli "What is the battery level?"
 *   tizenclaw-cli -s my_session "Run a skill"
 *   tizenclaw-cli --stream "Tell me about Tizen"
 *   tizenclaw-cli   (interactive mode)
 */

#include <arpa/inet.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>

#include <future>
#include <iostream>
#include <mutex>
#include <string>
#include <vector>

#include <nlohmann/json.hpp>

#include "tizenclaw.h"

namespace {

struct RequestContext {
  std::promise<std::string> promise;
  std::string response;
};

void OnResponseReady(const char* session_id, const char* response,
                     void* user_data) {
  (void)session_id;  // Unused in single-shot CLI
  auto* ctx = static_cast<RequestContext*>(user_data);
  if (ctx) {
    ctx->promise.set_value(response ? response : "");
  }
}

void OnStreamChunk(const char* session_id, const char* chunk, bool is_done,
                   void* user_data) {
  (void)session_id;  // Unused
  auto* ctx = static_cast<RequestContext*>(user_data);
  if (!ctx) return;

  if (chunk) {
    if (!is_done) {
      std::cout << chunk << std::flush;
    } else {
      ctx->response = chunk;
      std::cout << "\n";
    }
  }

  if (is_done) {
    ctx->promise.set_value(ctx->response);
  }
}

void OnErrorCallback(const char* session_id, int error_code,
                     const char* error_message, void* user_data) {
  (void)session_id;  // Unused
  auto* ctx = static_cast<RequestContext*>(user_data);
  if (!ctx) return;

  std::cerr << "\n[Error " << error_code << "] "
            << (error_message ? error_message : "Unknown error") << "\n";
  ctx->promise.set_value("");
}

std::string SendRequestThroughCAPI(tizenclaw_client_h client,
                                   const std::string& session_id,
                                   const std::string& prompt, bool stream) {
  RequestContext ctx;
  auto future = ctx.promise.get_future();

  int ret;
  if (stream) {
    ret = tizenclaw_client_send_request_stream(client, session_id.c_str(),
                                               prompt.c_str(), OnStreamChunk,
                                               OnErrorCallback, &ctx);
  } else {
    ret = tizenclaw_client_send_request(client, session_id.c_str(),
                                        prompt.c_str(), OnResponseReady,
                                        OnErrorCallback, &ctx);
  }

  if (ret != TIZENCLAW_ERROR_NONE) {
    std::cerr << "Failed to send request. Error code: " << ret << "\n";
    return "";
  }

  // Block until complete
  return future.get();
}

void PrintUsage() {
  std::cerr << "tizenclaw-cli — TizenClaw IPC test\n\n"
            << "Usage:\n"
            << "  tizenclaw-cli [options] [prompt]\n\n"
            << "Options:\n"
            << "  -s <id>       Session ID (default: cli_test)\n"
            << "  --stream      Enable streaming\n"
            << "  --send-to <channel> <text>\n"
            << "                Send outbound message via channel\n"
            << "  --list-agents List all running agents\n"
            << "  --perception  Show perception engine status\n"
            << "  -h, --help    Show this help\n\n"
            << "If no prompt given, interactive mode.\n";
}

// Direct IPC for send_to (bypasses CAPI)
int SendToChannel(const std::string& channel,
                  const std::string& text) {
  int sock = socket(AF_UNIX, SOCK_STREAM, 0);
  if (sock < 0) {
    std::cerr << "Failed to create socket\n";
    return 1;
  }

  struct sockaddr_un addr = {};
  addr.sun_family = AF_UNIX;
  const char kName[] = "tizenclaw.sock";
  for (size_t i = 0; i < sizeof(kName) - 1; ++i)
    addr.sun_path[1 + i] = kName[i];
  socklen_t addr_len =
      offsetof(struct sockaddr_un, sun_path)
      + 1 + sizeof(kName) - 1;

  if (connect(sock,
              reinterpret_cast<struct sockaddr*>(
                  &addr),
              addr_len) < 0) {
    close(sock);
    std::cerr << "Failed to connect to daemon\n";
    return 1;
  }

  // Build JSON-RPC request
  std::string req =
      "{\"jsonrpc\":\"2.0\",\"method\":"
      "\"send_to\",\"id\":1,\"params\":{"
      "\"channel\":\"" + channel + "\","
      "\"text\":\"";
  // Simple JSON escape
  for (char c : text) {
    if (c == '"') req += "\\\"";
    else if (c == '\\') req += "\\\\";
    else if (c == '\n') req += "\\n";
    else req += c;
  }
  req += "\"}}";

  uint32_t net_len = htonl(req.size());
  write(sock, &net_len, 4);
  write(sock, req.data(), req.size());

  // Read response
  uint32_t resp_len = 0;
  if (read(sock, &resp_len, 4) == 4) {
    resp_len = ntohl(resp_len);
    std::vector<char> buf(resp_len);
    size_t got = 0;
    while (got < resp_len) {
      auto r = read(sock, buf.data() + got,
                    resp_len - got);
      if (r <= 0) break;
      got += r;
    }
    std::cout << std::string(buf.data(), got)
              << "\n";
  }

  close(sock);
  return 0;
}

// Direct IPC for list_agents (bypasses CAPI)
int ListAgents() {
  int sock = socket(AF_UNIX, SOCK_STREAM, 0);
  if (sock < 0) {
    std::cerr << "Failed to create socket\n";
    return 1;
  }

  struct sockaddr_un addr = {};
  addr.sun_family = AF_UNIX;
  const char kName[] = "tizenclaw.sock";
  for (size_t i = 0; i < sizeof(kName) - 1; ++i)
    addr.sun_path[1 + i] = kName[i];
  socklen_t addr_len =
      offsetof(struct sockaddr_un, sun_path)
      + 1 + sizeof(kName) - 1;

  if (connect(sock,
              reinterpret_cast<struct sockaddr*>(
                  &addr),
              addr_len) < 0) {
    close(sock);
    std::cerr << "Failed to connect to daemon\n";
    return 1;
  }

  std::string req =
      "{\"jsonrpc\":\"2.0\",\"method\":"
      "\"list_agents\",\"id\":1,\"params\":{}}";

  uint32_t net_len = htonl(req.size());
  write(sock, &net_len, 4);
  write(sock, req.data(), req.size());

  // Read response
  uint32_t resp_len = 0;
  if (read(sock, &resp_len, 4) != 4) {
    close(sock);
    std::cerr << "Failed to read response\n";
    return 1;
  }
  resp_len = ntohl(resp_len);
  std::vector<char> buf(resp_len);
  size_t got = 0;
  while (got < resp_len) {
    auto r = read(sock, buf.data() + got,
                  resp_len - got);
    if (r <= 0) break;
    got += r;
  }
  close(sock);

  std::string body(buf.data(), got);

  // Parse and pretty-print
  try {
    auto j = nlohmann::json::parse(body);
    auto res = j.value("result",
                       nlohmann::json::object());

    // Configured roles
    if (res.contains("configured_roles")) {
      auto& roles = res["configured_roles"];
      std::cout << "=== Configured Roles ("
                << roles.size() << ") ===\n";
      for (auto& r : roles) {
        std::cout << "  - "
                  << r.value("name", "?")
                  << "  tools: ["
                  << r.value("allowed_tools",
                             nlohmann::json::array())
                         .dump()
                  << "]\n";
      }
    }

    // Dynamic agents
    if (res.contains("dynamic_agents") &&
        !res["dynamic_agents"].empty()) {
      auto& da = res["dynamic_agents"];
      std::cout << "\n=== Dynamic Agents ("
                << da.size() << ") ===\n";
      for (auto& a : da) {
        std::cout << "  - "
                  << a.value("name", "?") << "\n";
      }
    }

    // Active delegations
    if (res.contains("active_delegations")) {
      auto& del = res["active_delegations"];
      if (del.contains("active") &&
          !del["active"].empty()) {
        std::cout << "\n=== Active Delegations ("
                  << del["active"].size()
                  << ") ===\n";
        for (auto& d : del["active"]) {
          std::cout << "  - ["
                    << d.value("role", "?")
                    << "] " << d.value("task", "")
                    << " ("
                    << d.value("elapsed_sec", 0)
                    << "s)\n";
        }
      }
    }

    // Event bus sources
    if (res.contains("event_bus_sources") &&
        !res["event_bus_sources"].empty()) {
      auto& src = res["event_bus_sources"];
      std::cout << "\n=== Event Bus Sources ("
                << src.size() << ") ===\n";
      for (auto& s : src) {
        std::cout << "  - "
                  << s.value("name", "?")
                  << " (" << s.value("plugin_id", "")
                  << ")\n";
      }
    }

    // Autonomous trigger
    if (res.contains("autonomous_trigger")) {
      auto& at = res["autonomous_trigger"];
      std::cout << "\n=== Autonomous Trigger ==="
                << "\n  enabled: "
                << (at.value("enabled", false)
                        ? "yes" : "no")
                << "\n";
    }
  } catch (...) {
    // Fallback: raw JSON
    std::cout << body << "\n";
  }

  return 0;
}

// Direct IPC for perception status
int GetPerceptionStatus() {
  int sock = socket(AF_UNIX, SOCK_STREAM, 0);
  if (sock < 0) {
    std::cerr << "Failed to create socket\n";
    return 1;
  }

  struct sockaddr_un addr = {};
  addr.sun_family = AF_UNIX;
  const char kName[] = "tizenclaw.sock";
  for (size_t i = 0; i < sizeof(kName) - 1; ++i)
    addr.sun_path[1 + i] = kName[i];
  socklen_t addr_len =
      offsetof(struct sockaddr_un, sun_path)
      + 1 + sizeof(kName) - 1;

  if (connect(sock,
              reinterpret_cast<struct sockaddr*>(
                  &addr),
              addr_len) < 0) {
    close(sock);
    std::cerr << "Failed to connect to daemon\n";
    return 1;
  }

  std::string req =
      "{\"jsonrpc\":\"2.0\",\"method\":"
      "\"get_perception_status\","
      "\"id\":1,\"params\":{}}";

  uint32_t net_len = htonl(req.size());
  write(sock, &net_len, 4);
  write(sock, req.data(), req.size());

  // Read response
  uint32_t resp_len = 0;
  if (read(sock, &resp_len, 4) != 4) {
    close(sock);
    std::cerr << "Failed to read response\n";
    return 1;
  }
  resp_len = ntohl(resp_len);
  std::vector<char> buf(resp_len);
  size_t got = 0;
  while (got < resp_len) {
    auto r = read(sock, buf.data() + got,
                  resp_len - got);
    if (r <= 0) break;
    got += r;
  }
  close(sock);

  std::string body(buf.data(), got);

  // Parse and pretty-print
  try {
    auto j = nlohmann::json::parse(body);
    auto res = j.value("result",
                       nlohmann::json::object());

    // Engine status
    if (res.contains("engine")) {
      auto& e = res["engine"];
      std::cout << "=== Perception Engine ==="
                << "\n  Running: "
                << (e.value("running", false)
                        ? "yes" : "no")
                << "\n  Analysis interval: "
                << e.value(
                       "analysis_interval_sec", 0)
                << "s"
                << "\n  Events recorded: "
                << e.value("event_count", 0)
                << "\n";
    }

    // Situation assessment
    if (res.contains("situation")) {
      auto& s = res["situation"];
      std::string level =
          s.value("level", "unknown");
      std::string emoji = "✅";
      if (level == "advisory") emoji = "ℹ️ ";
      else if (level == "warning") emoji = "⚠️ ";
      else if (level == "critical") emoji = "🔴";

      std::cout << "\n=== Situation Assessment ==="
                << "\n  " << emoji << " Level: "
                << level
                << "\n  Risk Score: ";

      // Risk bar
      double risk = s.value("risk_score", 0.0);
      int pct = static_cast<int>(risk * 100);
      int filled = pct / 5;
      std::cout << "[";
      for (int i = 0; i < 20; i++) {
        std::cout << (i < filled ? "█" : "░");
      }
      std::cout << "] " << pct << "%\n";

      std::cout << "  Summary: "
                << s.value("summary", "") << "\n";

      if (s.contains("factors") &&
          !s["factors"].empty()) {
        std::cout << "\n  Risk Factors:\n";
        for (auto& f : s["factors"]) {
          std::cout << "    • " << f << "\n";
        }
      }
      if (s.contains("suggestions") &&
          !s["suggestions"].empty()) {
        std::cout << "\n  Suggestions:\n";
        for (auto& sg : s["suggestions"]) {
          std::cout << "    💡 " << sg << "\n";
        }
      }
    }

    // Device profile
    if (res.contains("profile")) {
      auto& p = res["profile"];
      std::cout << "\n=== Device Profile ==="
                << "\n  🔋 Battery: "
                << p.value("battery_level", -1)
                << "% ("
                << p.value("battery_health",
                           "unknown")
                << ")";
      if (p.value("charging", false)) {
        std::cout << " ⚡";
      }
      double drain = p.value(
          "battery_drain_rate", 0.0);
      if (drain > 0) {
        std::cout << "\n  📉 Drain rate: "
                  << drain << " %/min";
      }
      std::cout << "\n  🧠 Memory: "
                << p.value("memory_trend",
                           "unknown")
                << " ("
                << p.value(
                       "memory_warning_count", 0)
                << " warnings)"
                << "\n  🌐 Network: "
                << p.value("network_status",
                           "unknown")
                << " ("
                << p.value(
                       "network_drop_count", 0)
                << " drops)";

      auto fg = p.value("foreground_app", "");
      if (!fg.empty()) {
        std::cout << "\n  📱 Foreground: " << fg;
      }
      if (p.contains("top_apps") &&
          !p["top_apps"].empty()) {
        std::cout << "\n  📊 Top apps: ";
        bool first = true;
        for (auto& a : p["top_apps"]) {
          if (!first) std::cout << ", ";
          std::cout << a;
          first = false;
        }
      }
      std::cout << "\n";
    }

    // Anomalies
    if (res.contains("anomalies") &&
        !res["anomalies"].empty()) {
      std::cout << "\n=== Anomalies ===";
      for (auto& a : res["anomalies"]) {
        std::cout << "\n  ⚡ ["
                  << a.value("severity", "?")
                  << "] "
                  << a.value("type", "unknown")
                  << ": "
                  << a.value("detail", "");
      }
      std::cout << "\n";
    }

  } catch (...) {
    // Fallback: raw JSON
    std::cout << body << "\n";
  }

  return 0;
}

}  // namespace

int main(int argc, char* argv[]) {
  std::string session_id = "cli_test";
  bool stream = false;
  std::string prompt;

  for (int i = 1; i < argc; ++i) {
    std::string arg = argv[i];
    if (arg == "-h" || arg == "--help") {
      PrintUsage();
      return 0;
    } else if (arg == "--send-to" && i + 2 < argc) {
      std::string channel = argv[++i];
      std::string text;
      for (int j = ++i; j < argc; ++j) {
        if (!text.empty()) text += " ";
        text += argv[j];
      }
      return SendToChannel(channel, text);
    } else if (arg == "--list-agents") {
      return ListAgents();
    } else if (arg == "--perception") {
      return GetPerceptionStatus();
    } else if (arg == "-s" && i + 1 < argc) {
      session_id = argv[++i];
    } else if (arg == "--stream") {
      stream = true;
    } else {
      for (int j = i; j < argc; ++j) {
        if (!prompt.empty()) prompt += " ";
        prompt += argv[j];
      }
      break;
    }
  }

  tizenclaw_client_h client = nullptr;
  if (tizenclaw_client_create(&client) != TIZENCLAW_ERROR_NONE) {
    std::cerr << "Failed to create TizenClaw client.\n";
    return 1;
  }

  // Single-shot mode
  if (!prompt.empty()) {
    std::string resp =
        SendRequestThroughCAPI(client, session_id, prompt, stream);
    if (!stream && !resp.empty()) {
      std::cout << resp << "\n";
    }
    tizenclaw_client_destroy(client);
    return resp.empty() ? 1 : 0;
  }

  // Interactive mode
  std::cout << "TizenClaw CLI (session: " << session_id << ")\n"
            << "Type a prompt and press Enter. Ctrl+D to exit.\n\n";

  while (true) {
    std::cout << "you> " << std::flush;
    std::string line;
    if (!std::getline(std::cin, line)) break;
    if (line.empty()) continue;

    std::string resp = SendRequestThroughCAPI(client, session_id, line, stream);
    if (!stream && !resp.empty()) {
      std::cout << "\nassistant> " << resp << "\n\n";
    }
  }

  tizenclaw_client_destroy(client);
  std::cout << "\nBye.\n";
  return 0;
}
