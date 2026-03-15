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
#include "package_event_adapter.hh"

#include <app_info.h>
#include <app_manager.h>

#include <json.hpp>
#include <string>

#include "../../common/logging.hh"
#include "event_bus.hh"

namespace {

const char* EventTypeToString(
    package_manager_event_type_e type) {
  switch (type) {
    case PACKAGE_MANAGER_EVENT_TYPE_INSTALL:
      return "install";
    case PACKAGE_MANAGER_EVENT_TYPE_UNINSTALL:
      return "uninstall";
    case PACKAGE_MANAGER_EVENT_TYPE_UPDATE:
      return "update";
    case PACKAGE_MANAGER_EVENT_TYPE_MOVE:
      return "move";
    case PACKAGE_MANAGER_EVENT_TYPE_CLEAR:
      return "clear";
    default:
      return "unknown";
  }
}

const char* EventStateToString(
    package_manager_event_state_e state) {
  switch (state) {
    case PACKAGE_MANAGER_EVENT_STATE_STARTED:
      return "started";
    case PACKAGE_MANAGER_EVENT_STATE_PROCESSING:
      return "processing";
    case PACKAGE_MANAGER_EVENT_STATE_COMPLETED:
      return "completed";
    case PACKAGE_MANAGER_EVENT_STATE_FAILED:
      return "failed";
    default:
      return "unknown";
  }
}

}  // namespace

namespace tizenclaw {

PackageEventAdapter::~PackageEventAdapter() {
  Stop();
}

void PackageEventAdapter::Start() {
  if (started_) return;

  int ret = package_manager_create(&manager_);
  if (ret != PACKAGE_MANAGER_ERROR_NONE) {
    LOG(ERROR) << "PackageEventAdapter: "
               << "package_manager_create failed="
               << ret;
    return;
  }

  ret = package_manager_set_event_status(
      manager_,
      PACKAGE_MANAGER_STATUS_TYPE_ALL);
  if (ret != PACKAGE_MANAGER_ERROR_NONE) {
    LOG(ERROR) << "PackageEventAdapter: "
               << "set_event_status failed="
               << ret;
  }

  ret = package_manager_set_event_cb(
      manager_, OnPackageEvent, this);
  if (ret != PACKAGE_MANAGER_ERROR_NONE) {
    LOG(ERROR) << "PackageEventAdapter: "
               << "set_event_cb failed=" << ret;
    package_manager_destroy(manager_);
    manager_ = nullptr;
    return;
  }

  started_ = true;
  LOG(INFO) << "PackageEventAdapter: started";
}

void PackageEventAdapter::Stop() {
  if (!started_) return;

  if (manager_) {
    package_manager_unset_event_cb(manager_);
    package_manager_destroy(manager_);
    manager_ = nullptr;
  }

  started_ = false;
  LOG(INFO) << "PackageEventAdapter: stopped";
}

std::string PackageEventAdapter::GetName() const {
  return "PackageEventAdapter";
}

void PackageEventAdapter::OnPackageEvent(
    const char* type,
    const char* package,
    package_manager_event_type_e event_type,
    package_manager_event_state_e event_state,
    int progress,
    package_manager_error_e error,
    void* user_data) {
  if (!package) return;

  // Only publish on completed or failed state
  // to avoid flooding with progress events
  if (event_state !=
          PACKAGE_MANAGER_EVENT_STATE_COMPLETED &&
      event_state !=
          PACKAGE_MANAGER_EVENT_STATE_FAILED)
    return;

  SystemEvent ev;
  ev.type = EventType::kPackageChanged;
  ev.source = "package_manager";
  ev.plugin_id = "builtin";

  std::string event_type_str =
      EventTypeToString(event_type);
  ev.name = "package." + event_type_str;

  ev.data["event_type"] = event_type_str;
  ev.data["package_id"] = package;
  ev.data["package_type"] = type ? type : "unknown";
  ev.data["state"] =
      EventStateToString(event_state);
  ev.data["progress"] = progress;

  if (error != PACKAGE_MANAGER_ERROR_NONE)
    ev.data["error"] = static_cast<int>(error);

  // On install/update completion, query app info
  if (event_state ==
          PACKAGE_MANAGER_EVENT_STATE_COMPLETED &&
      event_type !=
          PACKAGE_MANAGER_EVENT_TYPE_UNINSTALL) {
    ev.data["apps"] = QueryAppInfo(package);
  }

  EventBus::GetInstance().Publish(std::move(ev));
}

nlohmann::json PackageEventAdapter::QueryAppInfo(
    const char* package_id) {
  auto apps = nlohmann::json::array();

  package_info_h pkg_info = nullptr;
  int ret = package_manager_get_package_info(
      package_id, &pkg_info);
  if (ret != PACKAGE_MANAGER_ERROR_NONE ||
      !pkg_info)
    return apps;

  // Iterate apps in the package
  package_info_foreach_app_from_package(
      pkg_info, PACKAGE_INFO_ALLAPP,
      [](package_info_app_component_type_e comp_type,
         const char* app_id,
         void* user_data) -> bool {
        auto* app_list =
            static_cast<nlohmann::json*>(user_data);
        if (!app_id) return true;

        nlohmann::json app_entry;
        app_entry["app_id"] = app_id;

        // Get app label and type
        app_info_h info = nullptr;
        if (app_info_create(app_id, &info) == 0 &&
            info) {
          char* label = nullptr;
          if (app_info_get_label(info, &label) == 0
              && label) {
            app_entry["label"] = label;
            free(label);
          }
          char* type = nullptr;
          if (app_info_get_type(info, &type) == 0
              && type) {
            app_entry["type"] = type;
            free(type);
          }
          app_info_destroy(info);
        }

        app_list->push_back(std::move(app_entry));
        return true;
      },
      &apps);

  package_info_destroy(pkg_info);
  return apps;
}

}  // namespace tizenclaw
