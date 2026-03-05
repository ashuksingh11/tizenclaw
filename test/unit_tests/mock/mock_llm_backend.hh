#ifndef __MOCK_LLM_BACKEND_H__
#define __MOCK_LLM_BACKEND_H__

#include <gmock/gmock.h>
#include "llm_backend.hh"

class MockLlmBackend : public LlmBackend {
public:
  MOCK_METHOD(bool, Initialize,
              (const nlohmann::json& config),
              (override));
  MOCK_METHOD(LlmResponse, Chat,
              (const std::vector<LlmMessage>& messages,
               const std::vector<LlmToolDecl>& tools),
              (override));
  MOCK_METHOD(std::string, GetName, (),
              (const, override));
  MOCK_METHOD(void, Shutdown, (),
              (override));
};

#endif  // __MOCK_LLM_BACKEND_H__
