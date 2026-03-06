#include "mcp_server.hh"
#include "agent_core.hh"
#include "../common/logging.hh"

#include <dirent.h>
#include <fstream>
#include <iostream>
#include <unistd.h>

namespace tizenclaw {


McpServer::McpServer(AgentCore* agent)
    : agent_(agent) {
  DiscoverTools();
}

void McpServer::DiscoverTools() {
  tools_.clear();

  // Scan skill manifests
  const std::string skills_dir =
      "/opt/usr/share/tizenclaw/skills";

  DIR* dir = opendir(skills_dir.c_str());
  if (dir) {
    struct dirent* ent;
    while ((ent = readdir(dir)) != nullptr) {
      if (ent->d_name[0] == '.') continue;
      std::string name(ent->d_name);
      if (name == "mcp_server") continue;

      std::string manifest_path =
          skills_dir + "/" + name + "/manifest.json";
      std::ifstream mf(manifest_path);
      if (!mf.is_open()) continue;

      try {
        nlohmann::json j;
        mf >> j;
        if (j.contains("parameters")) {
          ToolInfo t;
          t.name = j.value("name", name);
          t.description =
              j.value("description", "");
          t.input_schema = j["parameters"];
          t.is_skill = true;
          tools_.push_back(t);
          LOG(INFO) << "MCP: Discovered tool: "
                    << t.name;
        }
      } catch (...) {
        LOG(WARNING) << "MCP: Failed to parse: "
                     << manifest_path;
      }
    }
    closedir(dir);
  }

  // Add synthetic tool: ask_tizenclaw
  ToolInfo ask_tool;
  ask_tool.name = "ask_tizenclaw";
  ask_tool.description =
      "Send a natural language prompt to the "
      "TizenClaw AI Agent. The agent will plan "
      "and execute actions using available tools "
      "to fulfill the request.";
  ask_tool.input_schema = {
      {"type", "object"},
      {"properties", {
          {"prompt", {
              {"type", "string"},
              {"description",
               "The user's request in natural "
               "language"}
          }}
      }},
      {"required",
       nlohmann::json::array({"prompt"})}
  };
  ask_tool.is_skill = false;
  tools_.push_back(ask_tool);

  LOG(INFO) << "MCP: Total tools discovered: "
            << tools_.size();
}

void McpServer::RunStdio() {
  LOG(INFO) << "MCP Server started (stdio mode)";

  // Read line-delimited JSON-RPC from stdin
  std::string line;
  while (std::getline(std::cin, line)) {
    if (line.empty()) continue;

    try {
      auto request = nlohmann::json::parse(line);
      auto response = ProcessRequest(request);

      if (!response.is_null()) {
        std::cout << response.dump() << "\n";
        std::cout.flush();
      }
    } catch (const std::exception& e) {
      LOG(ERROR) << "MCP: JSON parse error: "
                 << e.what();
      // Send JSON-RPC parse error
      nlohmann::json err_resp = {
          {"jsonrpc", "2.0"},
          {"id", nullptr},
          {"error", {
              {"code", -32700},
              {"message", "Parse error"}
          }}
      };
      std::cout << err_resp.dump() << "\n";
      std::cout.flush();
    }
  }

  LOG(INFO) << "MCP Server stdio loop ended";
}

nlohmann::json McpServer::ProcessRequest(
    const nlohmann::json& request) {
  std::string method =
      request.value("method", "");
  auto params = request.value(
      "params", nlohmann::json::object());
  auto req_id = request.value(
      "id", nlohmann::json(nullptr));

  nlohmann::json result;

  if (method == "initialize") {
    result = HandleInitialize(params);
  } else if (method == "notifications/initialized") {
    // Notification — no response needed
    return nlohmann::json();
  } else if (method == "tools/list") {
    result = HandleToolsList(params);
  } else if (method == "tools/call") {
    result = HandleToolsCall(
        params, STDOUT_FILENO);
  } else {
    // Unknown method
    nlohmann::json response = {
        {"jsonrpc", "2.0"},
        {"id", req_id},
        {"error", {
            {"code", -32601},
            {"message", "Method not found"}
        }}
    };
    return response;
  }

  nlohmann::json response = {
      {"jsonrpc", "2.0"},
      {"id", req_id},
      {"result", result}
  };
  return response;
}

nlohmann::json McpServer::HandleInitialize(
    const nlohmann::json& /*params*/) {
  return {
      {"protocolVersion", kProtocolVersion},
      {"capabilities", {
          {"tools", nlohmann::json::object()}
      }},
      {"serverInfo", {
          {"name", "TizenClaw-MCP-Server"},
          {"version", kVersion}
      }}
  };
}

nlohmann::json McpServer::HandleToolsList(
    const nlohmann::json& /*params*/) {
  nlohmann::json tools_array =
      nlohmann::json::array();

  for (auto& t : tools_) {
    tools_array.push_back({
        {"name", t.name},
        {"description", t.description},
        {"inputSchema", t.input_schema}
    });
  }

  return {{"tools", tools_array}};
}

nlohmann::json McpServer::HandleToolsCall(
    const nlohmann::json& params,
    int /*stdout_fd*/) {
  std::string tool_name =
      params.value("name", "");
  auto arguments = params.value(
      "arguments", nlohmann::json::object());

  // Find the tool
  const ToolInfo* found = nullptr;
  for (auto& t : tools_) {
    if (t.name == tool_name) {
      found = &t;
      break;
    }
  }

  if (!found) {
    return {
        {"isError", true},
        {"content", nlohmann::json::array({
            {{"type", "text"},
             {"text", "Tool not found: " +
                      tool_name}}
        })}
    };
  }

  LOG(INFO) << "MCP: Calling tool: " << tool_name;

  if (!found->is_skill) {
    // ask_tizenclaw: route through Agentic Loop
    std::string prompt =
        arguments.value("prompt", "");
    if (prompt.empty()) {
      return {
          {"isError", true},
          {"content", nlohmann::json::array({
              {{"type", "text"},
               {"text", "Missing 'prompt' argument"}}
          })}
      };
    }

    std::string result =
        agent_->ProcessPrompt(
            "mcp_session", prompt);

    return {
        {"content", nlohmann::json::array({
            {{"type", "text"},
             {"text", result}}
        })}
    };
  }

  // Direct skill execution via container
  std::string result =
      agent_->ExecuteSkillForMcp(
          tool_name, arguments);

  return {
      {"content", nlohmann::json::array({
          {{"type", "text"},
           {"text", result}}
      })}
  };
}

} // namespace tizenclaw
