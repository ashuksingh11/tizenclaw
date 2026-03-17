/*
 * Copyright (c) 2026 Samsung Electronics Co., Ltd All Rights Reserved
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 */

#ifndef TIZENCLAW_TOOL_EXECUTOR_PYTHON_ENGINE_HH_
#define TIZENCLAW_TOOL_EXECUTOR_PYTHON_ENGINE_HH_

#include <mutex>
#include <string>
#include <utility>

namespace tizenclaw {
namespace tool_executor {

class PythonEngine {
 public:
  bool Initialize();
  bool IsInitialized() const { return initialized_; }
  std::pair<std::string, int> RunCode(const std::string& code);
  static std::string FindPython3();

 private:
  std::mutex mutex_;
  bool initialized_ = false;
};

}  // namespace tool_executor
}  // namespace tizenclaw

#endif  // TIZENCLAW_TOOL_EXECUTOR_PYTHON_ENGINE_HH_
