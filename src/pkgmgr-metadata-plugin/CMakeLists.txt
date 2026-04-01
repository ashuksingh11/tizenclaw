cmake_minimum_required(VERSION 3.12)

## ═══════════════════════════════════════════
##  Build Rust staticlib for metadata plugin core
## ═══════════════════════════════════════════

# The staticlib is built by the root `cargo_build` target (via Cargo workspace)
SET(RUST_STATICLIB "${CMAKE_SOURCE_DIR}/target/release/libtizenclaw_metadata_plugin.a")

## ═══════════════════════════════════════════
##  pkgmgr dependencies (declared once for all sub-plugins)
## ═══════════════════════════════════════════
find_package(PkgConfig REQUIRED)
pkg_check_modules(GLIB_DEPS REQUIRED glib-2.0)
pkg_check_modules(DLOG_DEPS REQUIRED dlog)
pkg_check_modules(PKGMGR_DEPS REQUIRED pkgmgr)
pkg_check_modules(PKGMGR_INFO_DEPS REQUIRED pkgmgr-info)
pkg_check_modules(PKGMGR_INSTALLER_DEPS REQUIRED pkgmgr-installer)
pkg_check_modules(PKGMGR_PARSER_DEPS REQUIRED pkgmgr-parser)

## ═══════════════════════════════════════════
##  Helper macro: build a metadata plugin .so
## ═══════════════════════════════════════════
macro(ADD_METADATA_PLUGIN TARGET_NAME PLUGIN_SUBDIR)
    ADD_LIBRARY(${TARGET_NAME} SHARED
        ${PLUGIN_SUBDIR}/plugin.c
    )

    ADD_DEPENDENCIES(${TARGET_NAME} cargo_build)

    TARGET_INCLUDE_DIRECTORIES(${TARGET_NAME} PRIVATE
        ${GLIB_DEPS_INCLUDE_DIRS}
        ${PKGMGR_PARSER_DEPS_INCLUDE_DIRS}
    )

    TARGET_LINK_LIBRARIES(${TARGET_NAME} PRIVATE
        ${RUST_STATICLIB}
        ${GLIB_DEPS_LIBRARIES}
        ${DLOG_DEPS_LIBRARIES}
        ${PKGMGR_DEPS_LIBRARIES}
        ${PKGMGR_INFO_DEPS_LIBRARIES}
        ${PKGMGR_INSTALLER_DEPS_LIBRARIES}
        ${PKGMGR_PARSER_DEPS_LIBRARIES}
    )

    # Set output library name with "lib" prefix
    SET_TARGET_PROPERTIES(${TARGET_NAME} PROPERTIES
        OUTPUT_NAME ${TARGET_NAME}
        PREFIX "lib"
    )

    # Install .so to pkgmgr parser metadata dir
    INSTALL(TARGETS ${TARGET_NAME}
        DESTINATION /etc/package-manager/parserlib/metadata
    )

    # Install .info to parser-plugins dir
    INSTALL(FILES ${PLUGIN_SUBDIR}/${TARGET_NAME}.info
        DESTINATION /usr/share/parser-plugins
    )
endmacro()

## ═══════════════════════════════════════════
##  Build all 3 metadata plugins
## ═══════════════════════════════════════════
ADD_METADATA_PLUGIN(tizenclaw-metadata-llm-backend-plugin llm-backend)
ADD_METADATA_PLUGIN(tizenclaw-metadata-skill-plugin skill)
ADD_METADATA_PLUGIN(tizenclaw-metadata-cli-plugin cli)
