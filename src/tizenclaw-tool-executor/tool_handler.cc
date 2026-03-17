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

#include "tool_handler.hh"

#include <fcntl.h>
#include <sys/wait.h>
#include <unistd.h>

#include <fstream>

#undef PROJECT_TAG
#define PROJECT_TAG "TIZENCLAW_TOOL_EXECUTOR"

#include "../common/logging.hh"

namespace tizenclaw {
namespace tool_executor {

namespace {

const std::string kAppDataDir = "/opt/usr/share/tizenclaw";
const std::vector<std::string> kToolSearchPaths = {
    kAppDataDir + "/tools/skills",
    kAppDataDir + "/tools/custom_skills",
    kAppDataDir + "/tools/cli",
};
constexpr int kExecTimeout = 30;
constexpr size_t kMaxPayload = 10 * 1024 * 1024;

std::string EscapeShellArg(const std::string& s) {
  std::string out = "'";
  for (char c : s) {
    if (c == '\'') out += "'\\''";
    else out += c;
  }
  out += "'";
  return out;
}

std::string ExtractJsonOutput(const std::string& raw) {
  auto output = raw;
  while (!output.empty() && output.back() == '\n') output.pop_back();
  auto pos = output.rfind('\n');
  std::string last_line = (pos != std::string::npos)
                              ? output.substr(pos + 1) : output;
  if (!last_line.empty() &&
      (last_line.front() == '{' || last_line.front() == '[')) {
    return last_line;
  }
  return output;
}

std::pair<std::string, int> RunCommand(const std::string& cmd,
                                        int timeout_sec = kExecTimeout) {
  int pipefd[2];
  if (pipe2(pipefd, O_CLOEXEC) == -1) return {"", -1};

  pid_t pid = fork();
  if (pid == -1) {
    close(pipefd[0]);
    close(pipefd[1]);
    return {"", -1};
  }

  if (pid == 0) {
    close(pipefd[0]);
    dup2(pipefd[1], STDOUT_FILENO);
    dup2(pipefd[1], STDERR_FILENO);
    close(pipefd[1]);
    const char* shell = "/bin/bash";
    if (access(shell, X_OK) != 0) shell = "/bin/sh";
    execl(shell, shell, "-c", cmd.c_str(), nullptr);
    _exit(127);
  }

  close(pipefd[1]);
  std::string output;
  char buf[4096];

  ssize_t n;
  while ((n = read(pipefd[0], buf, sizeof(buf))) > 0) {
    output.append(buf, n);
    if (output.size() > kMaxPayload) break;
  }
  close(pipefd[0]);

  int status = 0;
  waitpid(pid, &status, 0);
  int rc = WIFEXITED(status) ? WEXITSTATUS(status) : -1;
  return {output, rc};
}

}  // namespace

ToolHandler::ToolHandler(PythonEngine& python_engine)
    : python_engine_(python_engine) {}

std::pair<std::string, std::string> ToolHandler::DetectRuntime(
    const std::string& tool_name) {
  std::string runtime = "python";
  std::string entry_point = tool_name + ".py";

  for (const auto& base : kToolSearchPaths) {
    std::string manifest = base + "/" + tool_name + "/manifest.json";
    std::ifstream f(manifest);
    if (!f.is_open()) continue;
    try {
      nlohmann::json j;
      f >> j;
      runtime = j.value("runtime", "python");
      std::string ep;
      if (j.contains("entry_point"))
        ep = j["entry_point"].get<std::string>();
      else if (j.contains("entrypoint"))
        ep = j["entrypoint"].get<std::string>();
      if (!ep.empty()) {
        auto pos = ep.rfind(' ');
        entry_point = (pos != std::string::npos) ? ep.substr(pos + 1) : ep;
      } else {
        if (runtime == "python") entry_point = tool_name + ".py";
        else if (runtime == "node") entry_point = tool_name + ".js";
        else entry_point = tool_name;
      }
    } catch (...) {}
    break;
  }
  return {runtime, entry_point};
}

std::string ToolHandler::FindToolScript(const std::string& tool_name,
                                          const std::string& entry_point) {
  for (const auto& base : kToolSearchPaths) {
    std::string path = base + "/" + tool_name + "/" + entry_point;
    if (access(path.c_str(), R_OK) == 0) return path;
  }
  return "";
}

nlohmann::json ToolHandler::HandleTool(const std::string& tool_name,
                                         const std::string& args_str) {
  LOG(INFO) << "HandleTool: " << tool_name;

  auto [runtime, entry_point] = DetectRuntime(tool_name);
  std::string script = FindToolScript(tool_name, entry_point);
  if (script.empty()) {
    return {{"status", "error"},
            {"output", "Entry point not found for tool: " + tool_name}};
  }

  // For Python tools, try in-process execution
  if (runtime == "python" && python_engine_.IsInitialized()) {
    LOG(INFO) << "Executing Python tool in-process: " << script;

    std::ifstream f(script);
    if (!f.is_open()) {
      return {{"status", "error"},
              {"output", "Cannot open script: " + script}};
    }
    std::string code((std::istreambuf_iterator<char>(f)),
                      std::istreambuf_iterator<char>());

    std::string setup =
        "import os; os.environ['CLAW_ARGS'] = " +
        EscapeShellArg(args_str) + "\n";

    auto [output, rc] = python_engine_.RunCode(setup + code);
    if (rc != 0 && output.empty()) {
      return {{"status", "error"},
              {"output", "Python execution failed (rc=" +
                         std::to_string(rc) + ")"}};
    }
    if (rc != 0) {
      return {{"status", "error"},
              {"output", output.substr(0, 500)}};
    }
    return {{"status", "ok"}, {"output", ExtractJsonOutput(output)}};
  }

  // Fallback: fork/exec
  std::string cmd;
  if (runtime == "python") {
    std::string python = PythonEngine::FindPython3();
    if (python.empty()) {
      return {{"status", "error"}, {"output", "python3 not found"}};
    }
    cmd = "CLAW_ARGS=" + EscapeShellArg(args_str) +
          " " + python + " " + EscapeShellArg(script);
  } else if (runtime == "node") {
    cmd = "CLAW_ARGS=" + EscapeShellArg(args_str) +
          " /usr/bin/node " + EscapeShellArg(script);
  } else {
    cmd = "CLAW_ARGS=" + EscapeShellArg(args_str) +
          " " + EscapeShellArg(script);
  }

  LOG(INFO) << "Exec: runtime=" << runtime << " cmd=" << cmd;
  auto [output, rc] = RunCommand(cmd);

  if (rc != 0) {
    return {{"status", "error"},
            {"output", "exit " + std::to_string(rc) + ": " +
                       output.substr(0, 500)}};
  }
  return {{"status", "ok"}, {"output", ExtractJsonOutput(output)}};
}

}  // namespace tool_executor
}  // namespace tizenclaw
