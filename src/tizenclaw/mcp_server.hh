#ifndef __MCP_SERVER_H__
#define __MCP_SERVER_H__

#include <string>
#include <vector>
#include <json.hpp>

namespace tizenclaw {

class AgentCore;

class McpServer {
public:
    explicit McpServer(AgentCore* agent);

    // Run stdio JSON-RPC 2.0 loop (blocking).
    // Reads from stdin, writes to stdout. Logs to stderr.
    void RunStdio();

    // Process a single JSON-RPC 2.0 request and return
    // the response (or null json if notification).
    nlohmann::json ProcessRequest(
        const nlohmann::json& request);

private:
    // JSON-RPC 2.0 method handlers
    nlohmann::json HandleInitialize(
        const nlohmann::json& params);
    nlohmann::json HandleToolsList(
        const nlohmann::json& params);
    nlohmann::json HandleToolsCall(
        const nlohmann::json& params,
        int stdout_fd);

    // Discover tools from skill manifests
    void DiscoverTools();

    AgentCore* agent_;

    struct ToolInfo {
        std::string name;
        std::string description;
        nlohmann::json input_schema;
        bool is_skill = true;  // false for ask_tizenclaw
    };
    std::vector<ToolInfo> tools_;

    static constexpr const char* kVersion = "1.0.0";
    static constexpr const char* kProtocolVersion =
        "2024-11-05";
};

} // namespace tizenclaw

#endif // __MCP_SERVER_H__
