//! HTTP client using ureq (pure Rust).
//!
//! Provides GET/POST with retry and exponential backoff.

use serde_json::Value as JsonValue;

pub struct HttpResponse {
    pub status_code: u16,
    pub body: String,
    pub success: bool,
    pub error: String,
}

/// POST JSON to a URL with retry and backoff.
pub fn http_post(
    url: &str,
    headers: &[(&str, &str)],
    json_body: &str,
    max_retries: u32,
    timeout_secs: u64,
) -> HttpResponse {
    for attempt in 0..=max_retries {
        if attempt > 0 {
            let delay = std::time::Duration::from_millis(500 * (1 << (attempt - 1)));
            std::thread::sleep(delay);
        }
        match do_post(url, headers, json_body, timeout_secs) {
            Ok(resp) => return resp,
            Err(e) => {
                if attempt == max_retries {
                    return HttpResponse {
                        status_code: 0,
                        body: String::new(),
                        success: false,
                        error: e,
                    };
                }
            }
        }
    }
    unreachable!()
}

/// GET a URL with retry.
pub fn http_get(
    url: &str,
    headers: &[(&str, &str)],
    max_retries: u32,
    timeout_secs: u64,
) -> HttpResponse {
    for attempt in 0..=max_retries {
        if attempt > 0 {
            let delay = std::time::Duration::from_millis(500 * (1 << (attempt - 1)));
            std::thread::sleep(delay);
        }
        match do_get(url, headers, timeout_secs) {
            Ok(resp) => return resp,
            Err(e) => {
                if attempt == max_retries {
                    return HttpResponse {
                        status_code: 0,
                        body: String::new(),
                        success: false,
                        error: e,
                    };
                }
            }
        }
    }
    unreachable!()
}

fn do_post(url: &str, headers: &[(&str, &str)], body: &str, timeout_secs: u64) -> Result<HttpResponse, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build();

    let mut req = agent.post(url);
    for (k, v) in headers {
        req = req.set(k, v);
    }
    // Ensure Content-Type is set
    req = req.set("Content-Type", "application/json");

    match req.send_string(body) {
        Ok(resp) => {
            let status = resp.status();
            let body_str = resp.into_string().unwrap_or_default();
            Ok(HttpResponse {
                status_code: status,
                body: body_str,
                success: (200..300).contains(&status),
                error: String::new(),
            })
        }
        Err(ureq::Error::Status(code, resp)) => {
            let body_str = resp.into_string().unwrap_or_default();
            Ok(HttpResponse {
                status_code: code,
                body: body_str,
                success: false,
                error: format!("HTTP {}", code),
            })
        }
        Err(e) => Err(format!("Request failed: {}", e)),
    }
}

/// Convenience struct for channels/MCP.
pub struct HttpClient;

impl HttpClient {
    pub fn new() -> Self { HttpClient }

    pub fn get(&self, url: &str) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        let r = http_get(url, &[], 1, 30);
        if r.success { Ok(r) } else { Err(r.error.into()) }
    }

    pub fn post(&self, url: &str, body: &str) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        let r = http_post(url, &[], body, 1, 30);
        if r.success { Ok(r) } else { Err(r.error.into()) }
    }
}

fn do_get(url: &str, headers: &[(&str, &str)], timeout_secs: u64) -> Result<HttpResponse, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build();

    let mut req = agent.get(url);
    for (k, v) in headers {
        req = req.set(k, v);
    }

    match req.call() {
        Ok(resp) => {
            let status = resp.status();
            let body_str = resp.into_string().unwrap_or_default();
            Ok(HttpResponse {
                status_code: status,
                body: body_str,
                success: (200..300).contains(&status),
                error: String::new(),
            })
        }
        Err(ureq::Error::Status(code, resp)) => {
            let body_str = resp.into_string().unwrap_or_default();
            Ok(HttpResponse {
                status_code: code,
                body: body_str,
                success: false,
                error: format!("HTTP {}", code),
            })
        }
        Err(e) => Err(format!("Request failed: {}", e)),
    }
}
