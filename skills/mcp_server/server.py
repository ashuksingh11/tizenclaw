#!/usr/bin/env python3
import sys
import json
import os
import subprocess

# Local path to skills
SKILLS_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

def log(msg):
    # Log to stderr as stdout is used for JSON-RPC
    sys.stderr.write(f"[*] {msg}\n")
    sys.stderr.flush()

class McpServer:
    def __init__(self):
        self.tools = {}
        self.discover_tools()

    def discover_tools(self):
        log(f"Scanning skills in {SKILLS_DIR}")
        for skill_name in os.listdir(SKILLS_DIR):
            if skill_name == "mcp_server":
                continue
            
            manifest_path = os.path.join(SKILLS_DIR, skill_name, "manifest.json")
            if os.path.exists(manifest_path):
                try:
                    with open(manifest_path, "r") as f:
                        manifest = json.load(f)
                        tool_name = manifest.get("name", skill_name)
                        self.tools[tool_name] = {
                            "name": tool_name,
                            "description": manifest.get("description", ""),
                            "inputSchema": manifest.get("parameters", {"type": "object", "properties": {}}),
                            "skill_path": os.path.join(SKILLS_DIR, skill_name, f"{skill_name}.py")
                        }
                        log(f"Discovered tool: {tool_name}")
                except Exception as e:
                    log(f"Failed to load manifest for {skill_name}: {e}")

    def handle_request(self, request):
        method = request.get("method")
        params = request.get("params", {})
        req_id = request.get("id")

        if method == "initialize":
            return {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "TizenClaw-MCP-Server",
                    "version": "1.0.0"
                }
            }
        
        elif method == "notifications/initialized":
            return None # No response needed for notifications

        elif method == "tools/list":
            return {
                "tools": [
                    {
                        "name": t["name"],
                        "description": t["description"],
                        "inputSchema": t["inputSchema"]
                    } for t in self.tools.values()
                ]
            }

        elif method == "tools/call":
            tool_name = params.get("name")
            arguments = params.get("arguments", {})
            
            if tool_name not in self.tools:
                return {"isError": True, "content": [{"type": "text", "text": f"Tool {tool_name} not found"}]}
            
            tool = self.tools[tool_name]
            log(f"Calling tool {tool_name} with {arguments}")
            
            try:
                # Execute the skill script
                # We use CLAW_ARGS env var for input as per TizenClaw design
                env = os.environ.copy()
                env["CLAW_ARGS"] = json.dumps(arguments)
                
                result = subprocess.check_output(
                    [sys.executable, tool["skill_path"]],
                    env=env,
                    stderr=subprocess.PIPE,
                    timeout=30
                ).decode("utf-8")
                
                return {
                    "content": [{"type": "text", "text": result.strip()}]
                }
            except subprocess.CalledProcessError as e:
                return {
                    "isError": True,
                    "content": [{"type": "text", "text": f"Error: {e.stderr.decode('utf-8')}"}]
                }
            except Exception as e:
                return {
                    "isError": True,
                    "content": [{"type": "text", "text": f"Unexpected error: {str(e)}"}]
                }

        return {"error": {"code": -32601, "message": "Method not found"}}

    def run(self):
        log("TizenClaw MCP Server started (stdio mode)")
        for line in sys.stdin:
            try:
                request = json.loads(line)
                response_data = self.handle_request(request)
                
                if response_data is not None:
                    response = {
                        "jsonrpc": "2.0",
                        "id": request.get("id"),
                        "result": response_data
                    }
                    if "error" in response_data:
                        response.pop("result")
                        response["error"] = response_data["error"]
                    
                    sys.stdout.write(json.dumps(response) + "\n")
                    sys.stdout.flush()
            except Exception as e:
                log(f"Error processing request: {e}")

if __name__ == "__main__":
    server = McpServer()
    server.run()
