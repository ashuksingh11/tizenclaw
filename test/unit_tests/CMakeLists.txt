cmake_minimum_required(VERSION 3.5)

# Make sure we build unittests if not a subproject
if(NOT TARGET ${PACKAGE})
    project(tizenclaw-unittests CXX)
    set(CMAKE_CXX_STANDARD 17)
    set(CMAKE_CXX_STANDARD_REQUIRED ON)
endif()

find_package(PkgConfig REQUIRED)
pkg_check_modules(GTEST_DEPS REQUIRED gtest gmock)
pkg_check_modules(CORE_DEPS REQUIRED tizen-core glib-2.0 dlog libcurl sqlite3)
pkg_check_modules(LIBSOUP REQUIRED libsoup-2.4)
pkg_check_modules(LIBWEBSOCKETS REQUIRED libwebsockets)

# Test Sources
set(UNIT_TEST_SRCS
    main.cc
    agent_core_test.cc
    container_engine_test.cc
    telegram_client_test.cc
    llm_backend_test.cc
    http_client_test.cc
    session_store_test.cc
    mcp_server_test.cc
    tool_policy_test.cc
    key_store_test.cc
    audit_logger_test.cc
    task_scheduler_test.cc
    channel_test.cc
    skill_watcher_test.cc
    webhook_channel_test.cc
    agent_core_session_test.cc
    agent_role_test.cc
    pipeline_executor_test.cc
    a2a_handler_test.cc
    slack_discord_channel_test.cc
    embedding_store_test.cc
    web_dashboard_test.cc
    voice_channel_test.cc
)

# Mock Sources
set(MOCK_SRCS
    # mock/mock_dlog.cc
)

# Main project sources to test (excluding main())
set(CORE_SRCS
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/common/logging.cc
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/common/file_log_backend.cc
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/core/agent_core.cc
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/core/agent_role.cc
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/core/pipeline_executor.cc
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/core/tool_policy.cc
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/core/skill_watcher.cc
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/llm/llm_backend_factory.cc
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/llm/gemini_backend.cc
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/llm/openai_backend.cc
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/llm/anthropic_backend.cc
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/llm/ollama_backend.cc
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/channel/channel_registry.cc
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/channel/telegram_client.cc
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/channel/mcp_server.cc
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/channel/webhook_channel.cc
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/channel/slack_channel.cc
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/channel/discord_channel.cc
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/channel/web_dashboard.cc
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/channel/a2a_handler.cc
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/channel/voice_channel.cc
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/storage/session_store.cc
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/storage/embedding_store.cc
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/storage/audit_logger.cc
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/infra/http_client.cc
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/infra/key_store.cc
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/infra/container_engine.cc
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/scheduler/task_scheduler.cc
)

add_executable(tizenclaw-unittests
    ${UNIT_TEST_SRCS}
    ${MOCK_SRCS}
    ${CORE_SRCS}
)

target_include_directories(tizenclaw-unittests PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${CMAKE_CURRENT_SOURCE_DIR}/mock
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/core
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/llm
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/channel
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/storage
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/infra
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/tizenclaw/scheduler
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/common/nlohmann
    ${GTEST_DEPS_INCLUDE_DIRS}
    ${CORE_DEPS_INCLUDE_DIRS}
    ${LIBSOUP_INCLUDE_DIRS}
    ${LIBWEBSOCKETS_INCLUDE_DIRS}
)

target_link_libraries(tizenclaw-unittests PUBLIC
    ${GTEST_DEPS_LIBRARIES}
    ${CORE_DEPS_LIBRARIES}
    ${LIBSOUP_LIBRARIES}
    ${LIBWEBSOCKETS_LIBRARIES}
    pthread
)

# CTest integration
enable_testing()
add_test(NAME TizenClawTests COMMAND tizenclaw-unittests)

# Install test bin (optional, can be installed for runtime)
install(TARGETS tizenclaw-unittests DESTINATION /usr/bin)
