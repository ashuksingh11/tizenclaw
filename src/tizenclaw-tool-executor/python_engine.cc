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

#include <Python.h>

#include "python_engine.hh"

#include <unistd.h>

#include <fstream>
#include <sstream>

#undef PROJECT_TAG
#define PROJECT_TAG "TIZENCLAW_TOOL_EXECUTOR"

#include "../common/logging.hh"

namespace tizenclaw {
namespace tool_executor {

bool PythonEngine::Initialize() {
  std::lock_guard<std::mutex> lock(mutex_);
  if (initialized_) return true;

  Py_Initialize();
  if (!Py_IsInitialized()) {
    LOG(ERROR) << "Py_Initialize() failed";
    return false;
  }

  initialized_ = true;
  LOG(INFO) << "Python interpreter initialized (linked)";
  return true;
}

std::pair<std::string, int> PythonEngine::RunCode(const std::string& code) {
  std::lock_guard<std::mutex> lock(mutex_);
  if (!initialized_) return {"Python not initialized", -1};

  char out_path[] = "/tmp/tizenclaw_pyout_XXXXXX";
  int fd = mkstemp(out_path);
  if (fd < 0) return {"Failed to create temp file", -1};
  close(fd);

  std::string wrapper =
      "import sys as _sys, io as _io\n"
      "_orig_stdout, _orig_stderr = _sys.stdout, _sys.stderr\n"
      "_buf = _io.StringIO()\n"
      "_sys.stdout = _sys.stderr = _buf\n"
      "try:\n";

  std::istringstream iss(code);
  std::string line;
  while (std::getline(iss, line)) {
    wrapper += "    " + line + "\n";
  }

  wrapper +=
      "except Exception as _e:\n"
      "    print(f'Error: {_e}', file=_buf)\n"
      "finally:\n"
      "    _sys.stdout, _sys.stderr = _orig_stdout, _orig_stderr\n"
      "    with open('" + std::string(out_path) + "', 'w') as _f:\n"
      "        _f.write(_buf.getvalue())\n";

  int rc = PyRun_SimpleString(wrapper.c_str());

  std::string output;
  std::ifstream ifs(out_path);
  if (ifs.is_open()) {
    output.assign(std::istreambuf_iterator<char>(ifs),
                  std::istreambuf_iterator<char>());
  }
  unlink(out_path);

  return {output, rc};
}

std::string PythonEngine::FindPython3() {
  for (const auto& p : {"/usr/bin/python3", "/usr/local/bin/python3"}) {
    if (access(p, X_OK) == 0) return p;
  }
  return "";
}

}  // namespace tool_executor
}  // namespace tizenclaw
