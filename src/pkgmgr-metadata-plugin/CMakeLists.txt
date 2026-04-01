cmake_minimum_required(VERSION 3.12)

find_package(PkgConfig REQUIRED)
pkg_check_modules(GLIB_DEPS REQUIRED glib-2.0)
pkg_check_modules(DLOG_DEPS REQUIRED dlog)
pkg_check_modules(PKGMGR_DEPS REQUIRED pkgmgr)
pkg_check_modules(PKGMGR_INFO_DEPS REQUIRED pkgmgr-info)
pkg_check_modules(PKGMGR_INSTALLER_DEPS REQUIRED pkgmgr-installer)
pkg_check_modules(PKGMGR_PARSER_DEPS REQUIRED pkgmgr-parser)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

macro(ADD_METADATA_PLUGIN_CPP TARGET_NAME PLUGIN_DIR PLUGIN_PREFIX)
    ADD_LIBRARY(${TARGET_NAME} SHARED
        ${PLUGIN_DIR}/tizenclaw_metadata_${PLUGIN_PREFIX}_plugin.cc
        common/logging.cc
    )

    TARGET_INCLUDE_DIRECTORIES(${TARGET_NAME} PRIVATE
        ${GLIB_DEPS_INCLUDE_DIRS}
        ${DLOG_DEPS_INCLUDE_DIRS}
        ${PKGMGR_DEPS_INCLUDE_DIRS}
        ${PKGMGR_INFO_DEPS_INCLUDE_DIRS}
        ${PKGMGR_INSTALLER_DEPS_INCLUDE_DIRS}
        ${PKGMGR_PARSER_DEPS_INCLUDE_DIRS}
        ${CMAKE_CURRENT_SOURCE_DIR}/common
    )

    TARGET_LINK_LIBRARIES(${TARGET_NAME} PRIVATE
        ${GLIB_DEPS_LIBRARIES}
        ${DLOG_DEPS_LIBRARIES}
        ${PKGMGR_DEPS_LIBRARIES}
        ${PKGMGR_INFO_DEPS_LIBRARIES}
        ${PKGMGR_INSTALLER_DEPS_LIBRARIES}
        ${PKGMGR_PARSER_DEPS_LIBRARIES}
    )

    SET_TARGET_PROPERTIES(${TARGET_NAME} PROPERTIES
        OUTPUT_NAME ${TARGET_NAME}
        PREFIX "lib"
    )

    INSTALL(TARGETS ${TARGET_NAME}
        DESTINATION /etc/package-manager/parserlib/metadata
    )

    INSTALL(FILES ${PLUGIN_DIR}/${TARGET_NAME}.info
        DESTINATION /usr/share/parser-plugins
    )
endmacro()

ADD_METADATA_PLUGIN_CPP(tizenclaw-metadata-llm-backend-plugin llm-backend llm_backend)
ADD_METADATA_PLUGIN_CPP(tizenclaw-metadata-skill-plugin skill skill)
ADD_METADATA_PLUGIN_CPP(tizenclaw-metadata-cli-plugin cli cli)
