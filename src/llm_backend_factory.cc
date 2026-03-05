#include "llm_backend.hh"
#include "gemini_backend.hh"
#include "openai_backend.hh"
#include "anthropic_backend.hh"
#include "ollama_backend.hh"

#include <dlog.h>

#ifdef  LOG_TAG
#undef  LOG_TAG
#endif
#define LOG_TAG "TizenClaw_LlmFactory"

std::unique_ptr<LlmBackend>
LlmBackendFactory::Create(
    const std::string& name) {
  if (name == "gemini") {
    return std::make_unique<GeminiBackend>();
  }
  if (name == "openai" || name == "chatgpt") {
    return std::make_unique<OpenAiBackend>();
  }
  if (name == "xai" || name == "grok") {
    // xAI uses OpenAI-compatible API
    return std::make_unique<OpenAiBackend>();
  }
  if (name == "anthropic" || name == "claude") {
    return std::make_unique<AnthropicBackend>();
  }
  if (name == "ollama") {
    return std::make_unique<OllamaBackend>();
  }

  dlog_print(DLOG_ERROR, LOG_TAG,
             "Unknown LLM backend: %s",
             name.c_str());
  return nullptr;
}
