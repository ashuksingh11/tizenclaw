#ifndef TIZENCLAW_CHANNEL_TELEGRAM_CLIENT_H_
#define TIZENCLAW_CHANNEL_TELEGRAM_CLIENT_H_

#include <string>
#include <set>
#include <thread>
#include <atomic>

#include "channel.hh"

namespace tizenclaw {


// Forward declaration
class AgentCore;

class TelegramClient : public Channel {
public:
    explicit TelegramClient(AgentCore* agent);
    ~TelegramClient();

    // Channel interface
    std::string GetName() const override {
      return "telegram";
    }
    bool Start() override;
    void Stop() override;
    bool IsRunning() const override {
      return running_;
    }

private:
    // Main loop for fetching updates using long-polling
    void PollingLoop();

    // Async handler: processes a single message on a worker thread
    void HandleMessage(long chat_id,
                       const std::string& text);

    // Parses telegram_config.json
    bool LoadConfig();

    // Sends a message back to the user via Telegram API
    void SendMessage(long chat_id, const std::string& text);

    // Edits an existing message (returns true on success)
    bool EditMessage(long chat_id, long message_id,
                     const std::string& text);

    AgentCore* agent_;
    std::string bot_token_;
    std::set<long> allowed_chat_ids_;

    std::thread polling_thread_;
    std::atomic<bool> running_;
    long update_offset_ = 0;

    // Concurrency control for message handlers
    std::atomic<int> active_handlers_{0};
    static constexpr int kMaxConcurrentHandlers = 3;
};

} // namespace tizenclaw

#endif // TIZENCLAW_CHANNEL_TELEGRAM_CLIENT_H_
