/*
 * Copyright (c) 2026 Samsung Electronics Co., Ltd All Rights Reserved
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
#ifndef PACKAGE_EVENT_ADAPTER_HH
#define PACKAGE_EVENT_ADAPTER_HH

#include <package_manager.h>

#include <json.hpp>
#include <string>

#include "event_adapter.hh"

namespace tizenclaw {

// Wraps Tizen package_manager.h API.
// Monitors package install/uninstall/update events
// and queries app info via app_info.h.
class PackageEventAdapter : public IEventAdapter {
 public:
  PackageEventAdapter() = default;
  ~PackageEventAdapter() override;

  void Start() override;
  void Stop() override;
  [[nodiscard]] std::string GetName() const override;

 private:
  // Static callback from package_manager API
  static void OnPackageEvent(
      const char* type,
      const char* package,
      package_manager_event_type_e event_type,
      package_manager_event_state_e event_state,
      int progress,
      package_manager_error_e error,
      void* user_data);

  // Query app info for a given package
  static nlohmann::json QueryAppInfo(
      const char* package_id);

  package_manager_h manager_ = nullptr;
  bool started_ = false;
};

}  // namespace tizenclaw

#endif  // PACKAGE_EVENT_ADAPTER_HH
