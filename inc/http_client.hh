#ifndef __HTTP_CLIENT_H__
#define __HTTP_CLIENT_H__

#include <map>
#include <string>

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
      long request_timeout_sec = 30);
};

#endif  // __HTTP_CLIENT_H__
