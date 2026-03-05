#ifndef __OLLAMA_BACKEND_H__
#define __OLLAMA_BACKEND_H__

#include "llm_backend.hh"

class OllamaBackend : public LlmBackend {
public:
  bool Initialize(
      const nlohmann::json& config) override;
  LlmResponse Chat(
      const std::vector<LlmMessage>& messages,
      const std::vector<LlmToolDecl>& tools)
      override;
  std::string GetName() const override {
    return "ollama";
  }

private:
  nlohmann::json ToOllamaMessages(
      const std::vector<LlmMessage>& messages) const;
  nlohmann::json ToOllamaTools(
      const std::vector<LlmToolDecl>& tools) const;
  LlmResponse ParseOllamaResponse(
      const std::string& body) const;

  std::string model_;
  std::string endpoint_;
};

#endif  // __OLLAMA_BACKEND_H__
