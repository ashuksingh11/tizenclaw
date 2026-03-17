PROJECT(tizenclaw-tool-executor CXX)

SET(TOOL_EXECUTOR_SRCS
    tool_executor_main.cc
    ${CMAKE_SOURCE_DIR}/src/common/logging.cc
)

ADD_EXECUTABLE(tizenclaw-tool-executor ${TOOL_EXECUTOR_SRCS})

TARGET_INCLUDE_DIRECTORIES(tizenclaw-tool-executor PRIVATE
    ${CMAKE_SOURCE_DIR}/src/common/nlohmann
)

APPLY_PKG_CONFIG(tizenclaw-tool-executor PUBLIC DLOG_DEPS)

# dlopen is used to load libpython at runtime (no build-time python dep)
TARGET_LINK_LIBRARIES(tizenclaw-tool-executor PRIVATE dl pthread)

INSTALL(TARGETS tizenclaw-tool-executor DESTINATION ${BINDIR})
