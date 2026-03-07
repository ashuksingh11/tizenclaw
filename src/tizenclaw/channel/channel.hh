#ifndef TIZENCLAW_CHANNEL_CHANNEL_H_
#define TIZENCLAW_CHANNEL_CHANNEL_H_

#include <string>

namespace tizenclaw {

// Abstract interface for communication channels.
// Each channel (Telegram, MCP, future Slack/Discord)
// implements this interface and registers with the
// ChannelRegistry for lifecycle management.
class Channel {
public:
    virtual ~Channel() = default;

    // Human-readable channel name (e.g. "telegram")
    virtual std::string GetName() const = 0;

    // Initialize and start the channel.
    // Returns false if startup fails (e.g. missing
    // config). Non-fatal: daemon continues without
    // this channel.
    virtual bool Start() = 0;

    // Signal the channel to stop and clean up.
    virtual void Stop() = 0;

    // Whether the channel is currently active.
    virtual bool IsRunning() const = 0;
};

} // namespace tizenclaw

#endif // TIZENCLAW_CHANNEL_CHANNEL_H_
