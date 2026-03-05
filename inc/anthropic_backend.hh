#ifndef __ANTHROPIC_BACKEND_H__
#define __ANTHROPIC_BACKEND_H__

#include "llm_backend.hh"

class AnthropicBackend : public LlmBackend {
public:
  bool Initialize(
      const nlohmann::json& config) override;
  LlmResponse Chat(
      const std::vector<LlmMessage>& messages,
      const std::vector<LlmToolDecl>& tools)
      override;
  std::string GetName() const override {
    return "anthropic";
  }

private:
  nlohmann::json ToAnthropicMessages(
      const std::vector<LlmMessage>& messages) const;
  nlohmann::json ToAnthropicTools(
      const std::vector<LlmToolDecl>& tools) const;
  LlmResponse ParseAnthropicResponse(
      const std::string& body) const;

  std::string api_key_;
  std::string model_;
};

#endif  // __ANTHROPIC_BACKEND_H__
