#ifndef __CHANNEL_REGISTRY_H__
#define __CHANNEL_REGISTRY_H__

#include <memory>
#include <string>
#include <vector>

#include "channel.hh"

namespace tizenclaw {

// Manages the lifecycle of all registered channels.
// Channels are registered during daemon startup and
// started/stopped as a group.
class ChannelRegistry {
public:
    ChannelRegistry() = default;
    ~ChannelRegistry();

    // Takes ownership of a channel.
    void Register(std::unique_ptr<Channel> ch);

    // Start all registered channels.
    // Channels that fail to start are logged but
    // do not prevent other channels from starting.
    void StartAll();

    // Stop all running channels in reverse order.
    void StopAll();

    // Look up a channel by name (nullptr if not
    // found).
    Channel* Get(const std::string& name) const;

    // List names of all registered channels.
    std::vector<std::string> ListChannels() const;

    // Number of registered channels.
    size_t Size() const { return channels_.size(); }

private:
    std::vector<std::unique_ptr<Channel>> channels_;
};

} // namespace tizenclaw

#endif // __CHANNEL_REGISTRY_H__
