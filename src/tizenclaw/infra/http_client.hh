#ifndef TIZENCLAW_INFRA_HTTP_CLIENT_H_
#define TIZENCLAW_INFRA_HTTP_CLIENT_H_

#include <string>
#include <map>
#include <functional>

namespace tizenclaw {


struct HttpResponse {
  long status_code = 0;
  std::string body;
  bool success = false;
  std::string error;
};

class HttpClient {
public:
  // POST JSON with retry + exponential backoff.
  static HttpResponse Post(
      const std::string& url,
      const std::map<std::string, std::string>&
          headers,
      const std::string& json_body,
      int max_retries = 3,
      long connect_timeout_sec = 10,
      long request_timeout_sec = 30,
      std::function<void(const std::string&)> stream_cb = nullptr);

  // GET with retry + timeouts (for long polling)
  static HttpResponse Get(
      const std::string& url,
      const std::map<std::string, std::string>&
          headers = {},
      int max_retries = 3,
      long connect_timeout_sec = 10,
      long request_timeout_sec = 40);
};

} // namespace tizenclaw

#endif  // TIZENCLAW_INFRA_HTTP_CLIENT_H_
