/*
 * Copyright (c) 2026 Samsung Electronics Co., Ltd All Rights Reserved
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 */

#ifndef TIZENCLAW_TOOL_EXECUTOR_FILE_MANAGER_HH_
#define TIZENCLAW_TOOL_EXECUTOR_FILE_MANAGER_HH_

#include <json.hpp>

namespace tizenclaw {
namespace tool_executor {

class FileManager {
 public:
  nlohmann::json Handle(const nlohmann::json& req);
};

}  // namespace tool_executor
}  // namespace tizenclaw

#endif  // TIZENCLAW_TOOL_EXECUTOR_FILE_MANAGER_HH_
