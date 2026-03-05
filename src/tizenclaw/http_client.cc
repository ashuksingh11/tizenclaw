#include "http_client.hh"

#include <curl/curl.h>
#include <thread>
#include <chrono>
#include <unistd.h>

#include "../common/logging.hh"

namespace tizenclaw {


struct WriteContext {
  std::string* body;
  std::function<void(const std::string&)> stream_cb;
};

static size_t WriteCallback(
    void* contents, size_t size, size_t nmemb,
    void* userp) {
  size_t total_size = size * nmemb;
  WriteContext* ctx = static_cast<WriteContext*>(userp);
  std::string chunk((char*)contents, total_size);
  
  if (ctx->body) {
    ctx->body->append(chunk);
  }
  
  if (ctx->stream_cb) {
    ctx->stream_cb(chunk);
  }
  
  return total_size;
}

HttpResponse HttpClient::Post(
    const std::string& url,
    const std::map<std::string, std::string>& hdrs,
    const std::string& json_body,
    int max_retries,
    long connect_timeout_sec,
    long request_timeout_sec,
    std::function<void(const std::string&)> stream_cb) {
  HttpResponse result;

  for (int attempt = 0; attempt < max_retries;
       ++attempt) {
    if (attempt > 0) {
      int delay_ms = 1000 * (1 << (attempt - 1));
      LOG(WARNING) << "Retry " << attempt << " after " << delay_ms << "ms";
      std::this_thread::sleep_for(
          std::chrono::milliseconds(delay_ms));
    }

    result.body.clear();
    result.error.clear();

    CURL* curl = curl_easy_init();
    if (!curl) {
      result.error = "curl_easy_init() failed";
      continue;
    }

    curl_easy_setopt(curl, CURLOPT_URL,
                     url.c_str());

    struct curl_slist* header_list = nullptr;
    for (auto& [k, v] : hdrs) {
      std::string h = k + ": " + v;
      header_list =
          curl_slist_append(header_list, h.c_str());
    }
    if (header_list) {
      curl_easy_setopt(curl, CURLOPT_HTTPHEADER,
                       header_list);
    }

    curl_easy_setopt(curl, CURLOPT_POSTFIELDS,
                     json_body.c_str());
                     
    WriteContext write_ctx;
    write_ctx.body = &result.body;
    write_ctx.stream_cb = stream_cb;
    
    curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION,
                     WriteCallback);
    curl_easy_setopt(curl, CURLOPT_WRITEDATA,
                     &write_ctx);
    curl_easy_setopt(curl, CURLOPT_SSL_VERIFYPEER,
                     1L);
    curl_easy_setopt(curl, CURLOPT_SSL_VERIFYHOST,
                     2L);
    // Tizen system CA bundle path
    const char* ca_paths[] = {
        "/etc/ssl/certs/ca-certificates.crt",
        "/etc/ssl/ca-bundle.pem",
        "/etc/pki/tls/certs/ca-bundle.crt",
        "/usr/share/ca-certificates/ca-bundle.crt",
        nullptr
    };
    for (int i = 0; ca_paths[i]; ++i) {
      if (access(ca_paths[i], R_OK) == 0) {
        curl_easy_setopt(curl, CURLOPT_CAINFO,
                         ca_paths[i]);
        break;
      }
    }
    curl_easy_setopt(curl, CURLOPT_CONNECTTIMEOUT,
                     connect_timeout_sec);
    curl_easy_setopt(curl, CURLOPT_TIMEOUT,
                     request_timeout_sec);

    CURLcode res = curl_easy_perform(curl);
    curl_easy_getinfo(curl,
                      CURLINFO_RESPONSE_CODE,
                      &result.status_code);
    if (header_list) {
      curl_slist_free_all(header_list);
    }
    curl_easy_cleanup(curl);

    if (res != CURLE_OK) {
      result.error = curl_easy_strerror(res);
      LOG(ERROR) << "curl failed: " << result.error << " (" << (attempt + 1) << "/" << max_retries << ")";
      continue;
    }

    if (result.status_code == 429 ||
        result.status_code >= 500) {
      result.error = "HTTP " + std::to_string(result.status_code) + " (Retry limit)";
      LOG(WARNING) << "HTTP " << result.status_code << ", retry (" << (attempt + 1) << "/" << max_retries << ")";
      continue;
    }

    result.success =
        (result.status_code >= 200 &&
         result.status_code < 300);
    if (!result.success) {
      result.error = "HTTP " +
          std::to_string(result.status_code);
    }
    return result;
  }

  LOG(ERROR) << "All " << max_retries << " retries failed";
  result.success = false;
  return result;
}

HttpResponse HttpClient::Get(
    const std::string& url,
    const std::map<std::string, std::string>& hdrs,
    int max_retries,
    long connect_timeout_sec,
    long request_timeout_sec) {
  HttpResponse result;

  for (int attempt = 0; attempt < max_retries;
       ++attempt) {
    if (attempt > 0) {
      int delay_ms = 1000 * (1 << (attempt - 1));
      LOG(WARNING) << "Retry " << attempt << " after " << delay_ms << "ms";
      std::this_thread::sleep_for(
          std::chrono::milliseconds(delay_ms));
    }

    result.body.clear();
    result.error.clear();

    CURL* curl = curl_easy_init();
    if (!curl) {
      result.error = "curl_easy_init() failed";
      continue;
    }

    curl_easy_setopt(curl, CURLOPT_URL,
                     url.c_str());

    struct curl_slist* header_list = nullptr;
    for (auto& [k, v] : hdrs) {
      std::string h = k + ": " + v;
      header_list =
          curl_slist_append(header_list, h.c_str());
    }
    if (header_list) {
      curl_easy_setopt(curl, CURLOPT_HTTPHEADER,
                       header_list);
    }

    // Default is GET, so no CURLOPT_POSTFIELDS needed
    curl_easy_setopt(curl, CURLOPT_HTTPGET, 1L);

    WriteContext write_ctx;
    write_ctx.body = &result.body;
    write_ctx.stream_cb = nullptr;

    curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION,
                     WriteCallback);
    curl_easy_setopt(curl, CURLOPT_WRITEDATA,
                     &write_ctx);
    curl_easy_setopt(curl, CURLOPT_SSL_VERIFYPEER,
                     1L);
    curl_easy_setopt(curl, CURLOPT_SSL_VERIFYHOST,
                     2L);

    const char* ca_paths[] = {
        "/etc/ssl/certs/ca-certificates.crt",
        "/etc/ssl/ca-bundle.pem",
        "/etc/pki/tls/certs/ca-bundle.crt",
        "/usr/share/ca-certificates/ca-bundle.crt",
        nullptr
    };
    for (int i = 0; ca_paths[i]; ++i) {
      if (access(ca_paths[i], R_OK) == 0) {
        curl_easy_setopt(curl, CURLOPT_CAINFO,
                         ca_paths[i]);
        break;
      }
    }
    curl_easy_setopt(curl, CURLOPT_CONNECTTIMEOUT,
                     connect_timeout_sec);
    curl_easy_setopt(curl, CURLOPT_TIMEOUT,
                     request_timeout_sec);

    CURLcode res = curl_easy_perform(curl);
    curl_easy_getinfo(curl,
                      CURLINFO_RESPONSE_CODE,
                      &result.status_code);
    if (header_list) {
      curl_slist_free_all(header_list);
    }
    curl_easy_cleanup(curl);

    if (res != CURLE_OK) {
      result.error = curl_easy_strerror(res);
      LOG(ERROR) << "curl failed: " << result.error << " (" << (attempt + 1) << "/" << max_retries << ")";
      continue;
    }

    if (result.status_code == 429 ||
        result.status_code >= 500) {
      result.error = "HTTP " + std::to_string(result.status_code) + " (Retry limit)";
      LOG(WARNING) << "HTTP " << result.status_code << ", retry (" << (attempt + 1) << "/" << max_retries << ")";
      continue;
    }

    result.success =
        (result.status_code >= 200 &&
         result.status_code < 300);
    if (!result.success) {
      result.error = "HTTP " +
          std::to_string(result.status_code);
    }
    return result;
  }

  LOG(ERROR) << "All " << max_retries << " retries failed";
  result.success = false;
  return result;
}

} // namespace tizenclaw
