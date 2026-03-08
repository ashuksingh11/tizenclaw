#ifndef TIZENCLAW_INFRA_HEALTH_MONITOR_H_
#define TIZENCLAW_INFRA_HEALTH_MONITOR_H_

#include <string>
#include <atomic>
#include <chrono>

namespace tizenclaw {

// Collects system health metrics for
// monitoring and dashboard display.
class HealthMonitor {
 public:
  HealthMonitor();
  ~HealthMonitor() = default;

  // Increment counters (thread-safe)
  void IncrementRequestCount();
  void IncrementErrorCount();
  void IncrementLlmCallCount();
  void IncrementToolCallCount();

  // Get all metrics as JSON string
  std::string GetMetricsJson() const;

  // Get individual metrics
  uint64_t GetRequestCount() const;
  uint64_t GetErrorCount() const;
  uint64_t GetLlmCallCount() const;
  uint64_t GetToolCallCount() const;
  double GetUptimeSeconds() const;

 private:
  // Parse memory from /proc/self/status
  void ParseMemoryInfo(
      int& rss_kb, int& vm_kb) const;

  // Parse CPU load from /proc/loadavg
  void ParseCpuLoad(
      double& l1, double& l5,
      double& l15) const;

  // Thread count from /proc/self/status
  int GetThreadCount() const;

  // Counters
  std::atomic<uint64_t> request_count_{0};
  std::atomic<uint64_t> error_count_{0};
  std::atomic<uint64_t> llm_call_count_{0};
  std::atomic<uint64_t> tool_call_count_{0};

  // Start time
  std::chrono::steady_clock::time_point
      start_time_;
};

}  // namespace tizenclaw

#endif  // TIZENCLAW_INFRA_HEALTH_MONITOR_H_
