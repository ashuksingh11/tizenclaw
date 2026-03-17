PROJECT(tizenclaw-tool-executor CXX)

SET(TOOL_EXECUTOR_SRCS
    tool_executor_main.cc
    ${CMAKE_SOURCE_DIR}/src/common/logging.cc
)

ADD_EXECUTABLE(tizenclaw-tool-executor ${TOOL_EXECUTOR_SRCS})

# Find Python3 include/lib paths from sysconfig
EXECUTE_PROCESS(
    COMMAND python3 -c "import sysconfig; print(sysconfig.get_config_var('INCLUDEPY'))"
    OUTPUT_VARIABLE PYTHON3_INCLUDE_DIR
    OUTPUT_STRIP_TRAILING_WHITESPACE
)
EXECUTE_PROCESS(
    COMMAND python3 -c "import sysconfig; print(sysconfig.get_config_var('LIBDIR'))"
    OUTPUT_VARIABLE PYTHON3_LIB_DIR
    OUTPUT_STRIP_TRAILING_WHITESPACE
)
EXECUTE_PROCESS(
    COMMAND python3 -c "import sysconfig; v=sysconfig.get_config_var('VERSION'); print(f'python{v}')"
    OUTPUT_VARIABLE PYTHON3_LIB_NAME
    OUTPUT_STRIP_TRAILING_WHITESPACE
)

MESSAGE(STATUS "Python3 include: ${PYTHON3_INCLUDE_DIR}")
MESSAGE(STATUS "Python3 libdir:  ${PYTHON3_LIB_DIR}")
MESSAGE(STATUS "Python3 lib:     ${PYTHON3_LIB_NAME}")

TARGET_INCLUDE_DIRECTORIES(tizenclaw-tool-executor PRIVATE
    ${CMAKE_SOURCE_DIR}/src/common/nlohmann
    ${PYTHON3_INCLUDE_DIR}
)

APPLY_PKG_CONFIG(tizenclaw-tool-executor PUBLIC DLOG_DEPS)

TARGET_LINK_DIRECTORIES(tizenclaw-tool-executor PRIVATE ${PYTHON3_LIB_DIR})
TARGET_LINK_LIBRARIES(tizenclaw-tool-executor PRIVATE ${PYTHON3_LIB_NAME} pthread)

INSTALL(TARGETS tizenclaw-tool-executor DESTINATION ${BINDIR})
