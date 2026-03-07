#include "web_dashboard.hh"
#include "agent_core.hh"
#include "task_scheduler.hh"
#include "audit_logger.hh"
#include "../common/logging.hh"

#include <fstream>
#include <sstream>
#include <cstring>
#include <sys/stat.h>

namespace tizenclaw {

WebDashboard::WebDashboard(
    AgentCore* agent,
    TaskScheduler* scheduler)
    : agent_(agent),
      scheduler_(scheduler) {
  web_root_ =
      std::string(APP_DATA_DIR) + "/web";
}

WebDashboard::~WebDashboard() {
  Stop();
}

bool WebDashboard::LoadConfig() {
  std::string config_path =
      std::string(APP_DATA_DIR) +
      "/config/dashboard_config.json";
  std::ifstream f(config_path);
  if (f.is_open()) {
    try {
      nlohmann::json j;
      f >> j;
      port_ = j.value("port", 9090);
      web_root_ = j.value(
          "web_root", web_root_);
    } catch (const std::exception& e) {
      LOG(WARNING)
          << "Failed to parse dashboard "
          << "config: " << e.what();
    }
  }

  // Check web_root exists
  struct stat st;
  if (stat(web_root_.c_str(), &st) != 0 ||
      !S_ISDIR(st.st_mode)) {
    LOG(WARNING)
        << "Web root not found: "
        << web_root_;
    return false;
  }
  return true;
}

void WebDashboard::HandleRequest(
    SoupServer* /*server*/,
    SoupMessage* msg,
    const char* path,
    GHashTable* /*query*/,
    SoupClientContext* /*client*/,
    gpointer user_data) {
  auto* self =
      static_cast<WebDashboard*>(user_data);

  // Add CORS headers
  SoupMessageHeaders* resp_headers =
      msg->response_headers;
  soup_message_headers_append(
      resp_headers,
      "Access-Control-Allow-Origin", "*");
  soup_message_headers_append(
      resp_headers,
      "Access-Control-Allow-Methods",
      "GET, POST, OPTIONS");
  soup_message_headers_append(
      resp_headers,
      "Access-Control-Allow-Headers",
      "Content-Type");

  // Handle OPTIONS (CORS preflight)
  if (msg->method == SOUP_METHOD_OPTIONS) {
    soup_message_set_status(
        msg, SOUP_STATUS_OK);
    return;
  }

  std::string req_path(path);

  // Route API requests
  if (req_path.substr(0, 5) == "/api/") {
    self->HandleApi(msg, req_path);
    return;
  }

  // Serve static files
  self->ServeStaticFile(msg, req_path);
}

void WebDashboard::HandleApi(
    SoupMessage* msg,
    const std::string& path) const {
  if (path == "/api/status") {
    ApiStatus(msg);
  } else if (path == "/api/sessions") {
    ApiSessions(msg);
  } else if (path == "/api/tasks") {
    ApiTasks(msg);
  } else if (path == "/api/logs") {
    ApiLogs(msg);
  } else if (path == "/api/chat") {
    ApiChat(msg);
  } else {
    soup_message_set_status(
        msg, SOUP_STATUS_NOT_FOUND);
    soup_message_set_response(
        msg, "application/json",
        SOUP_MEMORY_COPY,
        "{\"error\":\"Not found\"}", 21);
  }
}

void WebDashboard::ServeStaticFile(
    SoupMessage* msg,
    const std::string& path) const {
  std::string file_path = web_root_;

  if (path == "/" || path.empty()) {
    file_path += "/index.html";
  } else {
    // Prevent directory traversal
    if (path.find("..") != std::string::npos) {
      soup_message_set_status(
          msg, SOUP_STATUS_FORBIDDEN);
      return;
    }
    file_path += path;
  }

  std::ifstream f(file_path, std::ios::binary);
  if (!f.is_open()) {
    soup_message_set_status(
        msg, SOUP_STATUS_NOT_FOUND);
    soup_message_set_response(
        msg, "text/html",
        SOUP_MEMORY_COPY,
        "<h1>404 Not Found</h1>", 22);
    return;
  }

  std::string content(
      (std::istreambuf_iterator<char>(f)),
      std::istreambuf_iterator<char>());

  // Determine MIME type
  std::string content_type = "text/html";
  if (path.size() >= 4) {
    std::string ext =
        path.substr(path.rfind('.'));
    if (ext == ".css") {
      content_type = "text/css";
    } else if (ext == ".js") {
      content_type =
          "application/javascript";
    } else if (ext == ".json") {
      content_type = "application/json";
    } else if (ext == ".png") {
      content_type = "image/png";
    } else if (ext == ".svg") {
      content_type = "image/svg+xml";
    }
  }

  soup_message_set_status(
      msg, SOUP_STATUS_OK);
  soup_message_set_response(
      msg, content_type.c_str(),
      SOUP_MEMORY_COPY,
      content.c_str(),
      static_cast<gsize>(content.size()));
}

void WebDashboard::ApiStatus(
    SoupMessage* msg) const {
  nlohmann::json status = {
      {"status", "running"},
      {"version", "1.0.0"},
      {"channels",
       agent_ ? "active" : "inactive"}
  };
  std::string body = status.dump();
  soup_message_set_status(
      msg, SOUP_STATUS_OK);
  soup_message_set_response(
      msg, "application/json",
      SOUP_MEMORY_COPY,
      body.c_str(),
      static_cast<gsize>(body.size()));
}

void WebDashboard::ApiSessions(
    SoupMessage* msg) const {
  // List session files from sessions directory
  nlohmann::json sessions =
      nlohmann::json::array();

  std::string sessions_dir =
      std::string(APP_DATA_DIR) + "/sessions";
  DIR* dir = opendir(sessions_dir.c_str());
  if (dir) {
    struct dirent* ent;
    while ((ent = readdir(dir)) != nullptr) {
      if (ent->d_name[0] == '.') continue;
      std::string name = ent->d_name;
      if (name.size() > 3 &&
          name.substr(name.size() - 3) ==
              ".md") {
        std::string id =
            name.substr(0, name.size() - 3);

        // Get file info
        std::string fpath =
            sessions_dir + "/" + name;
        struct stat st;
        stat(fpath.c_str(), &st);

        sessions.push_back({
            {"id", id},
            {"file", name},
            {"size_bytes", st.st_size},
            {"modified", st.st_mtime}
        });
      }
    }
    closedir(dir);
  }

  std::string body = sessions.dump();
  soup_message_set_status(
      msg, SOUP_STATUS_OK);
  soup_message_set_response(
      msg, "application/json",
      SOUP_MEMORY_COPY,
      body.c_str(),
      static_cast<gsize>(body.size()));
}

void WebDashboard::ApiTasks(
    SoupMessage* msg) const {
  // List task files from tasks directory
  nlohmann::json tasks =
      nlohmann::json::array();

  std::string tasks_dir =
      std::string(APP_DATA_DIR) + "/tasks";
  DIR* dir = opendir(tasks_dir.c_str());
  if (dir) {
    struct dirent* ent;
    while ((ent = readdir(dir)) != nullptr) {
      if (ent->d_name[0] == '.') continue;
      std::string name = ent->d_name;
      if (name.size() > 3 &&
          name.substr(name.size() - 3) ==
              ".md") {
        // Read task file for metadata
        std::string fpath =
            tasks_dir + "/" + name;
        std::ifstream tf(fpath);
        std::string content;
        if (tf.is_open()) {
          content.assign(
              (std::istreambuf_iterator<char>(
                   tf)),
              std::istreambuf_iterator<char>());
        }

        tasks.push_back({
            {"file", name},
            {"content_preview",
             content.substr(0, 200)}
        });
      }
    }
    closedir(dir);
  }

  std::string body = tasks.dump();
  soup_message_set_status(
      msg, SOUP_STATUS_OK);
  soup_message_set_response(
      msg, "application/json",
      SOUP_MEMORY_COPY,
      body.c_str(),
      static_cast<gsize>(body.size()));
}

void WebDashboard::ApiLogs(
    SoupMessage* msg) const {
  // Read today's audit log
  nlohmann::json logs = nlohmann::json::array();

  // Get today's date
  auto now = std::chrono::system_clock::now();
  auto t = std::chrono::system_clock::to_time_t(
      now);
  struct tm tm_buf;
  localtime_r(&t, &tm_buf);
  char date_buf[16];
  std::strftime(date_buf, sizeof(date_buf),
                "%Y-%m-%d", &tm_buf);
  std::string date(date_buf);

  std::string log_path =
      std::string(APP_DATA_DIR) +
      "/audit/" + date + ".md";
  std::ifstream lf(log_path);
  if (lf.is_open()) {
    std::string content(
        (std::istreambuf_iterator<char>(lf)),
        std::istreambuf_iterator<char>());

    // Return last 2000 chars of the log
    size_t start = 0;
    if (content.size() > 2000) {
      start = content.size() - 2000;
    }
    logs.push_back({
        {"date", date},
        {"content",
         content.substr(start)}
    });
  }

  std::string body = logs.dump();
  soup_message_set_status(
      msg, SOUP_STATUS_OK);
  soup_message_set_response(
      msg, "application/json",
      SOUP_MEMORY_COPY,
      body.c_str(),
      static_cast<gsize>(body.size()));
}

void WebDashboard::ApiChat(
    SoupMessage* msg) const {
  // Only accept POST
  if (msg->method != SOUP_METHOD_POST) {
    soup_message_set_status(
        msg, SOUP_STATUS_METHOD_NOT_ALLOWED);
    return;
  }

  // Extract body
  SoupMessageBody* body = msg->request_body;
  std::string payload;
  if (body && body->data &&
      body->length > 0) {
    payload.assign(
        body->data, body->length);
  }

  std::string prompt;
  std::string session_id = "web_dashboard";
  try {
    auto j = nlohmann::json::parse(payload);
    prompt = j.value("prompt", "");
    session_id = j.value(
        "session_id", "web_dashboard");
  } catch (...) {
    prompt = payload;
  }

  if (prompt.empty() || !agent_) {
    soup_message_set_status(
        msg, SOUP_STATUS_BAD_REQUEST);
    soup_message_set_response(
        msg, "application/json",
        SOUP_MEMORY_COPY,
        "{\"error\":\"Empty prompt\"}", 24);
    return;
  }

  std::string result =
      agent_->ProcessPrompt(
          session_id, prompt);

  nlohmann::json resp = {
      {"status", "ok"},
      {"session_id", session_id},
      {"response", result}
  };
  std::string resp_str = resp.dump();

  soup_message_set_status(
      msg, SOUP_STATUS_OK);
  soup_message_set_response(
      msg, "application/json",
      SOUP_MEMORY_COPY,
      resp_str.c_str(),
      static_cast<gsize>(resp_str.size()));
}

bool WebDashboard::Start() {
  if (running_) return true;

  if (!LoadConfig()) {
    LOG(WARNING)
        << "WebDashboard: no web root, "
        << "skipping";
    return false;
  }

  GError* error = nullptr;
  server_ = soup_server_new(
      SOUP_SERVER_SERVER_HEADER,
      "TizenClaw-Dashboard",
      nullptr);

  if (!server_) {
    LOG(ERROR) << "Failed to create "
               << "dashboard SoupServer";
    return false;
  }

  // Register handler for all paths
  soup_server_add_handler(
      server_, "/",
      HandleRequest, this, nullptr);

  // Listen on configured port
  if (!soup_server_listen_all(
          server_, port_,
          static_cast<SoupServerListenOptions>(0),
          &error)) {
    LOG(ERROR) << "Dashboard: failed to listen "
               << "on port " << port_
               << ": " << error->message;
    g_error_free(error);
    g_object_unref(server_);
    server_ = nullptr;
    return false;
  }

  running_ = true;

  // Run GMainLoop in a separate thread
  server_thread_ = std::thread([this]() {
    loop_ = g_main_loop_new(nullptr, FALSE);
    LOG(INFO) << "Web dashboard running on "
              << "port " << port_;
    g_main_loop_run(loop_);
    g_main_loop_unref(loop_);
    loop_ = nullptr;
  });

  LOG(INFO) << "WebDashboard started on "
            << "port " << port_;
  return true;
}

void WebDashboard::Stop() {
  if (!running_) return;

  running_ = false;

  if (loop_) {
    g_main_loop_quit(loop_);
  }

  if (server_thread_.joinable()) {
    server_thread_.join();
  }

  if (server_) {
    soup_server_disconnect(server_);
    g_object_unref(server_);
    server_ = nullptr;
  }

  LOG(INFO) << "WebDashboard stopped.";
}

}  // namespace tizenclaw
