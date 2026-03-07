#ifndef TIZENCLAW_LLM_OLLAMA_BACKEND_H_
#define TIZENCLAW_LLM_OLLAMA_BACKEND_H_

#include "llm_backend.hh"

namespace tizenclaw {


class OllamaBackend : public LlmBackend {
public:
  bool Initialize(
      const nlohmann::json& config) override;
  LlmResponse Chat(
      const std::vector<LlmMessage>& messages,
      const std::vector<LlmToolDecl>& tools,
      std::function<void(const std::string&)> on_chunk = nullptr,
      const std::string& system_prompt = "")
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

} // namespace tizenclaw

#endif  // TIZENCLAW_LLM_OLLAMA_BACKEND_H_
