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

#include "file_manager.hh"

#include <filesystem>
#include <fstream>
#include <string>
#include <vector>

#undef PROJECT_TAG
#define PROJECT_TAG "TIZENCLAW_TOOL_EXECUTOR"

#include "../common/logging.hh"

namespace tizenclaw {
namespace tool_executor {

namespace {

const std::string kAppDataDir = "/opt/usr/share/tizenclaw";
const std::vector<std::string> kAllowedPaths = {
    kAppDataDir + "/tools/custom_skills",
    kAppDataDir + "/data",
};

}  // namespace

nlohmann::json FileManager::Handle(const nlohmann::json& req) {
  namespace fs = std::filesystem;
  std::string operation = req.value("operation", "");
  std::string path = req.value("path", "");

  if (path.empty()) {
    return {{"status", "error"}, {"output", "No path provided"}};
  }

  std::error_code ec;
  std::string real = fs::canonical(path, ec).string();
  if (ec) real = path;

  bool allowed = false;
  for (const auto& prefix : kAllowedPaths) {
    if (real.starts_with(prefix + "/") || real == prefix) {
      allowed = true;
      break;
    }
  }
  if (!allowed) {
    return {{"status", "error"},
            {"output", "Path outside allowed directories"}};
  }

  LOG(INFO) << "FileManager: op=" << operation << " path=" << path;

  try {
    if (operation == "write_file") {
      std::string content = req.value("content", "");
      fs::create_directories(fs::path(path).parent_path(), ec);
      std::ofstream f(path);
      if (!f.is_open())
        return {{"status", "error"}, {"output", "Failed to write file"}};
      f << content;
      nlohmann::json r = {{"result", "file_written"},
                          {"path", path}, {"size", (int)content.size()}};
      return {{"status", "ok"}, {"output", r.dump()}};
    }
    if (operation == "read_file") {
      if (!fs::is_regular_file(path, ec))
        return {{"status", "error"}, {"output", "File not found: " + path}};
      std::ifstream f(path);
      std::string content((std::istreambuf_iterator<char>(f)),
                           std::istreambuf_iterator<char>());
      nlohmann::json r = {{"result", "file_read"}, {"path", path},
                          {"content", content}, {"size", (int)content.size()}};
      return {{"status", "ok"}, {"output", r.dump()}};
    }
    if (operation == "delete_file") {
      if (!fs::exists(path, ec))
        return {{"status", "error"}, {"output", "Not found: " + path}};
      fs::remove_all(path, ec);
      nlohmann::json r = {{"result", "deleted"}, {"path", path}};
      return {{"status", "ok"}, {"output", r.dump()}};
    }
    if (operation == "list_dir") {
      if (!fs::is_directory(path, ec))
        return {{"status", "error"}, {"output", "Not a directory: " + path}};
      nlohmann::json entries = nlohmann::json::array();
      for (const auto& e : fs::directory_iterator(path, ec)) {
        entries.push_back({
            {"name", e.path().filename().string()},
            {"type", e.is_directory() ? "dir" : "file"},
            {"size", e.is_regular_file() ? (int)e.file_size() : 0},
        });
      }
      nlohmann::json r = {{"result", "listing"},
                          {"path", path}, {"entries", entries}};
      return {{"status", "ok"}, {"output", r.dump()}};
    }
  } catch (const std::exception& e) {
    return {{"status", "error"},
            {"output", std::string("file_manager error: ") + e.what()}};
  }

  return {{"status", "error"},
          {"output", "Unknown operation: " + operation}};
}

}  // namespace tool_executor
}  // namespace tizenclaw
