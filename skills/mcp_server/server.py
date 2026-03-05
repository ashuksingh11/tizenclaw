#!/usr/bin/env python3
import sys
import json
import os
import subprocess
import socket
import struct

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
        self.add_tizenclaw_tool()

    def add_tizenclaw_tool(self):
        self.tools["ask_tizenclaw"] = {
            "name": "ask_tizenclaw",
            "description": "Send a prompt directly to the TizenClaw LLM Agent",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "prompt": {"type": "string", "description": "The user's request"}
                },
                "required": ["prompt"]
            },
            "is_synthetic": True
        }

    def _ask_tizenclaw(self, prompt: str):
        req = {
            "session_id": "mcp_session",
            "text": prompt,
            "stream": True
        }
        payload = json.dumps(req).encode('utf-8')
        
        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        try:
            sock.connect("\0tizenclaw.ipc")
            
            # Send length-prefixed payload
            length_prefix = struct.pack("!I", len(payload))
            sock.sendall(length_prefix + payload)
            
            # Read streaming response
            full_text = ""
            while True:
                # Read length prefix
                resp_len_data = sock.recv(4)
                if len(resp_len_data) != 4:
                    yield {"error": "Failed to receive IPC response length"}
                    break
                    
                resp_len = struct.unpack("!I", resp_len_data)[0]
                
                # Read payload
                resp_data = bytearray()
                while len(resp_data) < resp_len:
                    chunk = sock.recv(min(4096, resp_len - len(resp_data)))
                    if not chunk:
                        break
                    resp_data.extend(chunk)
                    
                if len(resp_data) != resp_len:
                    yield {"error": "Incomplete IPC response"}
                    break
                    
                resp_json = json.loads(resp_data.decode('utf-8'))
                
                if resp_json.get("type") == "stream_chunk":
                    chunk_text = resp_json.get("text", "")
                    full_text += chunk_text
                    yield {"chunk": chunk_text}
                elif resp_json.get("type") in ["response", "stream_end"]:
                    full_text = resp_json.get("text", full_text)
                    yield {"text": full_text}
                    break
                else:
                    yield {"error": f"Unknown response type: {resp_json.get('type')}"}
                    break
                    
        except Exception as e:
            yield {"error": f"Error communicating with TizenClaw daemon: {e}"}
        finally:
            sock.close()

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
            
            if tool.get("is_synthetic"):
                if tool_name == "ask_tizenclaw":
                    meta = params.get("_meta", {})
                    progress_token = meta.get("progressToken")
                    
                    final_text = ""
                    error_text = None
                    for chunk_map in self._ask_tizenclaw(arguments.get("prompt", "")):
                        if "error" in chunk_map:
                            error_text = chunk_map["error"]
                            break
                        if "chunk" in chunk_map:
                            if progress_token:
                                progress_req = {
                                    "jsonrpc": "2.0",
                                    "method": "$/progress",
                                    "params": {
                                        "progressToken": progress_token,
                                        "progress": len(final_text),
                                        "total": None,
                                        "data": chunk_map["chunk"]
                                    }
                                }
                                sys.stdout.write(json.dumps(progress_req) + "\n")
                                sys.stdout.flush()
                        if "text" in chunk_map:
                            final_text = chunk_map["text"]
                    
                    if error_text:
                        return {"isError": True, "content": [{"type": "text", "text": error_text}]}

                    return {
                        "content": [{"type": "text", "text": final_text}]
                    }
            
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
