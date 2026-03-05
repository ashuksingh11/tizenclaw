#ifndef __TELEGRAM_CLIENT_H__
#define __TELEGRAM_CLIENT_H__

#include <string>
#include <set>
#include <thread>
#include <atomic>

namespace tizenclaw {


// Forward declaration
class AgentCore;

class TelegramClient {
public:
    explicit TelegramClient(AgentCore* agent);
    ~TelegramClient();

    // Loads config and starts the background polling thread
    bool Start();

    // Signals the thread to stop and joins it
    void Stop();

private:
    // Main loop for fetching updates using long-polling
    void PollingLoop();

    // Parses telegram_config.json
    bool LoadConfig();

    // Sends a message back to the user via Telegram API
    void SendMessage(long chat_id, const std::string& text);

    AgentCore* agent_;
    std::string bot_token_;
    std::set<long> allowed_chat_ids_;

    std::thread polling_thread_;
    std::atomic<bool> running_;
    long update_offset_ = 0;
};

} // namespace tizenclaw

#endif // __TELEGRAM_CLIENT_H__
