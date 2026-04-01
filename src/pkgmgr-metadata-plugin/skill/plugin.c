/*
 * Copyright (c) 2026 Samsung Electronics Co., Ltd.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#include <glib.h>
#include <pkgmgr_parser.h>

#define EXPORT __attribute__((visibility("default")))

/* Rust FFI — implemented in libtizenclaw_metadata_plugin.a */
extern int tizenclaw_metadata_check_privilege(
    const char* pkgid, GList* metadata,
    const char* metadata_key, const char* plugin_name);

static const char* METADATA_KEY = "http://tizen.org/metadata/tizenclaw/skill";
static const char* PLUGIN_NAME = "skill";

EXPORT int PKGMGR_MDPARSER_PLUGIN_INSTALL(const char* pkgid,
                                           const char* appid,
                                           GList* metadata) {
    (void)appid;
    return tizenclaw_metadata_check_privilege(pkgid, metadata, METADATA_KEY, PLUGIN_NAME);
}

EXPORT int PKGMGR_MDPARSER_PLUGIN_UPGRADE(const char* pkgid,
                                           const char* appid,
                                           GList* metadata) {
    (void)appid;
    return tizenclaw_metadata_check_privilege(pkgid, metadata, METADATA_KEY, PLUGIN_NAME);
}

EXPORT int PKGMGR_MDPARSER_PLUGIN_UNINSTALL(const char* pkgid,
                                             const char* appid,
                                             GList* metadata) {
    (void)pkgid; (void)appid; (void)metadata;
    return 0;
}

EXPORT int PKGMGR_MDPARSER_PLUGIN_CLEAN(const char* pkgid,
                                         const char* appid,
                                         GList* metadata) {
    (void)pkgid; (void)appid; (void)metadata;
    return 0;
}

EXPORT int PKGMGR_MDPARSER_PLUGIN_UNDO(const char* pkgid,
                                        const char* appid,
                                        GList* metadata) {
    (void)pkgid; (void)appid; (void)metadata;
    return 0;
}

EXPORT int PKGMGR_MDPARSER_PLUGIN_REMOVED(const char* pkgid,
                                           const char* appid,
                                           GList* metadata) {
    (void)pkgid; (void)appid; (void)metadata;
    return 0;
}

EXPORT int PKGMGR_MDPARSER_PLUGIN_RECOVERINSTALL(const char* pkgid,
                                                   const char* appid,
                                                   GList* metadata) {
    return PKGMGR_MDPARSER_PLUGIN_INSTALL(pkgid, appid, metadata);
}

EXPORT int PKGMGR_MDPARSER_PLUGIN_RECOVERUPGRADE(const char* pkgid,
                                                   const char* appid,
                                                   GList* metadata) {
    return PKGMGR_MDPARSER_PLUGIN_UPGRADE(pkgid, appid, metadata);
}

EXPORT int PKGMGR_MDPARSER_PLUGIN_RECOVERUNINSTALL(const char* pkgid,
                                                     const char* appid,
                                                     GList* metadata) {
    (void)pkgid; (void)appid; (void)metadata;
    return 0;
}
