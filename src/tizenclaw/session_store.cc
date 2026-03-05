#include <fstream>
#include <sstream>
#include <sys/stat.h>

#include "session_store.hh"
#include "../common/logging.hh"

namespace tizenclaw {


SessionStore::SessionStore()
    : sessions_dir_(
          "/opt/usr/share/tizenclaw/sessions") {
}

void SessionStore::SetDirectory(
    const std::string& dir) {
  sessions_dir_ = dir;
}

std::string SessionStore::GetSessionPath(
    const std::string& session_id) const {
  return sessions_dir_ + "/" + session_id + ".json";
}

nlohmann::json SessionStore::MessageToJson(
    const LlmMessage& msg) {
  nlohmann::json j;
  j["role"] = msg.role;

  if (!msg.text.empty()) {
    j["text"] = msg.text;
  }

  if (!msg.tool_calls.empty()) {
    nlohmann::json tcs = nlohmann::json::array();
    for (auto& tc : msg.tool_calls) {
      tcs.push_back({
          {"id", tc.id},
          {"name", tc.name},
          {"args", tc.args}
      });
    }
    j["tool_calls"] = tcs;
  }

  if (!msg.tool_name.empty()) {
    j["tool_name"] = msg.tool_name;
  }

  if (!msg.tool_call_id.empty()) {
    j["tool_call_id"] = msg.tool_call_id;
  }

  if (!msg.tool_result.is_null()) {
    j["tool_result"] = msg.tool_result;
  }

  return j;
}

LlmMessage SessionStore::JsonToMessage(
    const nlohmann::json& j) {
  LlmMessage msg;
  msg.role = j.value("role", "");
  msg.text = j.value("text", "");
  msg.tool_name = j.value("tool_name", "");
  msg.tool_call_id = j.value("tool_call_id", "");

  if (j.contains("tool_result")) {
    msg.tool_result = j["tool_result"];
  }

  if (j.contains("tool_calls")) {
    for (auto& tc : j["tool_calls"]) {
      LlmToolCall call;
      call.id = tc.value("id", "");
      call.name = tc.value("name", "");
      if (tc.contains("args")) {
        call.args = tc["args"];
      }
      msg.tool_calls.push_back(call);
    }
  }

  return msg;
}

bool SessionStore::SaveSession(
    const std::string& session_id,
    const std::vector<LlmMessage>& history) {
  if (session_id.empty() || history.empty()) {
    return false;
  }

  // Ensure directory exists
  mkdir(sessions_dir_.c_str(), 0700);

  nlohmann::json arr = nlohmann::json::array();
  for (auto& msg : history) {
    arr.push_back(MessageToJson(msg));
  }

  std::string data = arr.dump(2);

  // Check file size limit — trim oldest messages
  while (data.size() > kMaxFileSize &&
         arr.size() > 2) {
    arr.erase(arr.begin());
    data = arr.dump(2);
  }

  std::string path = GetSessionPath(session_id);
  std::ofstream out(path);
  if (!out.is_open()) {
    LOG(ERROR) << "Failed to save session: " << path;
    return false;
  }

  out << data;
  out.close();

  LOG(DEBUG) << "Session saved: " << session_id << " (" << arr.size() << " messages, " << data.size() << " bytes)";
  return true;
}

std::vector<LlmMessage> SessionStore::LoadSession(
    const std::string& session_id) {
  std::vector<LlmMessage> history;

  std::string path = GetSessionPath(session_id);
  std::ifstream in(path);
  if (!in.is_open()) {
    return history;  // No saved session
  }

  try {
    nlohmann::json arr;
    in >> arr;
    in.close();

    if (!arr.is_array()) {
      LOG(WARNING) << "Invalid session file: " << path;
      return history;
    }

    for (auto& j : arr) {
      history.push_back(JsonToMessage(j));
    }

    LOG(INFO) << "Session loaded: " << session_id << " (" << history.size() << " messages)";
  } catch (const std::exception& e) {
    LOG(ERROR) << "Failed to parse session " << path << ": " << e.what();
    history.clear();
  }

  return history;
}

void SessionStore::DeleteSession(
    const std::string& session_id) {
  std::string path = GetSessionPath(session_id);
  if (remove(path.c_str()) == 0) {
    LOG(INFO) << "Session deleted: " << session_id;
  }
}

} // namespace tizenclaw
