/*
 * Copyright (c) 2026 Samsung Electronics Co., Ltd
 * All Rights Reserved
 *
 * Licensed under the Apache License, Version 2.0
 */

#ifndef __SKILL_WATCHER_H__
#define __SKILL_WATCHER_H__

#include <atomic>
#include <functional>
#include <map>
#include <mutex>
#include <string>
#include <thread>

namespace tizenclaw {

// Watches /opt/usr/share/tizenclaw/skills/ for
// manifest.json changes using Linux inotify.
// Calls a user-provided callback when skills
// need reloading.
class SkillWatcher {
 public:
  using ReloadCallback = std::function<void()>;

  SkillWatcher();
  ~SkillWatcher();

  // Start watching the given directory.
  // callback is invoked when manifest changes
  // are detected (debounced 500ms).
  bool Start(const std::string& skills_dir,
             ReloadCallback callback);

  // Stop watching and join the thread.
  void Stop();

  bool IsRunning() const {
    return running_.load();
  }

 private:
  // Watch thread entry point
  void WatchLoop();

  // Add inotify watch for a subdirectory
  void AddSubdirWatch(const std::string& path);

  // Scan and add watches for all subdirs
  void ScanSubdirectories();

  int inotify_fd_ = -1;
  std::string skills_dir_;
  ReloadCallback callback_;
  std::atomic<bool> running_{false};
  std::thread watch_thread_;

  // Map: watch descriptor -> directory path
  std::map<int, std::string> wd_to_path_;
  std::mutex wd_mutex_;
};

}  // namespace tizenclaw

#endif  // __SKILL_WATCHER_H__
