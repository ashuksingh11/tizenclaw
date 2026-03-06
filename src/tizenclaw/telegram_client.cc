#include "telegram_client.hh"
#include "agent_core.hh"
#include "http_client.hh"

#include "../common/logging.hh"
#include <fstream>
#include <chrono>
#include <iostream>

namespace tizenclaw {


TelegramClient::TelegramClient(AgentCore* agent)
    : agent_(agent), running_(false) {
}

TelegramClient::~TelegramClient() {
    Stop();
}

bool TelegramClient::LoadConfig() {
    std::string config_path =
        "/opt/usr/share/tizenclaw/config/"
        "telegram_config.json";
    std::ifstream f(config_path);
    if (!f.is_open()) {
        LOG(WARNING) << "No telegram_config.json found";
        return false;
    }

    try {
        nlohmann::json j;
        f >> j;
        bot_token_ = j.value("bot_token", "");
        
        if (j.contains("allowed_chat_ids") &&
            j["allowed_chat_ids"].is_array()) {
            for (auto& id : j["allowed_chat_ids"]) {
                allowed_chat_ids_.insert(
                    id.get<long>());
            }
        }
    } catch (const std::exception& e) {
        LOG(ERROR) << "Failed to parse config: " << e.what();
        return false;
    }

    if (bot_token_.empty() ||
        bot_token_ == "YOUR_TELEGRAM_BOT_TOKEN_HERE") {
        LOG(WARNING) << "Invalid or default BOT_TOKEN.";
        return false;
    }

    return true;
}

bool TelegramClient::Start() {
    if (running_) {
        return true;
    }

    if (!LoadConfig()) {
        return false;
    }

    running_ = true;
    polling_thread_ =
        std::thread(&TelegramClient::PollingLoop,
                    this);
    LOG(INFO) << "TelegramClient started polling.";
    return true;
}

void TelegramClient::Stop() {
    if (running_) {
        running_ = false;
        if (polling_thread_.joinable()) {
            polling_thread_.join();
        }
        LOG(INFO) << "TelegramClient stopped.";
    }
}

void TelegramClient::SendMessage(
    long chat_id, const std::string& text) {
    if (bot_token_.empty()) return;

    std::string url =
        "https://api.telegram.org/bot" +
        bot_token_ + "/sendMessage";

    // Truncate to Telegram's 4096 char limit
    std::string safe_text = text;
    if (safe_text.length() > 4000) {
        safe_text = safe_text.substr(0, 4000) +
                    "\n...(truncated)";
    }

    nlohmann::json payload = {
        {"chat_id", chat_id},
        {"text", safe_text},
        {"parse_mode", "Markdown"}
    };

    auto resp = HttpClient::Post(
        url,
        {{"Content-Type", "application/json"}},
        payload.dump());

    if (!resp.success) {
        LOG(WARNING) << "SendMessage Markdown parse failed, retrying as plain text";
        payload.erase("parse_mode");
        resp = HttpClient::Post(
            url,
            {{"Content-Type", "application/json"}},
            payload.dump());
        
        if (!resp.success) {
            LOG(ERROR) << "SendMessage failed: " << resp.error;
        }
    }
}

void TelegramClient::PollingLoop() {
    std::string url =
        "https://api.telegram.org/bot" + bot_token_;
    
    while (running_) {
        std::string req_url = url +
            "/getUpdates?offset=" +
            std::to_string(update_offset_) +
            "&timeout=30";

        // Call HTTP GET with a 40 second timeout
        // (to allow for 30s long polling + network)
        auto resp = HttpClient::Get(
            req_url, {}, 1, 10, 40);

        if (!running_) break;

        if (!resp.success) {
            LOG(ERROR) << "Polling network error: " << resp.error;
            std::this_thread::sleep_for(
                std::chrono::seconds(5));
            continue;
        }

        try {
            auto j = nlohmann::json::parse(resp.body);
            if (!j.value("ok", false)) {
                LOG(ERROR) << "API returned not ok";
                std::this_thread::sleep_for(
                    std::chrono::seconds(5));
                continue;
            }

            for (auto& item : j["result"]) {
                update_offset_ =
                    item["update_id"].get<long>() + 1;

                if (!item.contains("message")) {
                    continue;
                }

                auto msg = item["message"];
                if (!msg.contains("text") ||
                    !msg.contains("chat")) {
                    continue;
                }

                std::string text =
                    msg.value("text", "");
                long chat_id =
                    msg["chat"].value("id", 0L);

                if (text.empty() || chat_id == 0) {
                    continue;
                }

                // Apply allowlist filter
                if (!allowed_chat_ids_.empty() &&
                    allowed_chat_ids_.find(chat_id) ==
                        allowed_chat_ids_.end()) {
                    LOG(INFO) << "Blocked chat_id " << chat_id << " - not in allowlist";
                    continue;
                }

                LOG(INFO) << "Received from " << chat_id << ": " << text;

                // Directly invoke AgentCore bypassing IPC
                std::string session_id =
                    "telegram_" + std::to_string(chat_id);
                std::string response =
                    agent_->ProcessPrompt(session_id,
                                          text);

                SendMessage(chat_id, response);
            }
        } catch (const std::exception& e) {
            LOG(ERROR) << "Polling JSON error: " << e.what();
            std::this_thread::sleep_for(
                std::chrono::seconds(5));
        }
    }
}

} // namespace tizenclaw
