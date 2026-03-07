#include "channel_registry.hh"
#include "../../common/logging.hh"

namespace tizenclaw {

ChannelRegistry::~ChannelRegistry() {
  StopAll();
}

void ChannelRegistry::Register(
    std::unique_ptr<Channel> ch) {
  if (!ch) return;
  LOG(INFO) << "Channel registered: "
            << ch->GetName();
  channels_.push_back(std::move(ch));
}

void ChannelRegistry::StartAll() {
  for (auto& ch : channels_) {
    if (!ch->Start()) {
      LOG(WARNING)
          << "Channel failed to start: "
          << ch->GetName()
          << " (continuing without it)";
    } else {
      LOG(INFO) << "Channel started: "
                << ch->GetName();
    }
  }
}

void ChannelRegistry::StopAll() {
  // Stop in reverse registration order
  for (auto it = channels_.rbegin();
       it != channels_.rend(); ++it) {
    if ((*it)->IsRunning()) {
      LOG(INFO) << "Stopping channel: "
                << (*it)->GetName();
      (*it)->Stop();
    }
  }
}

Channel* ChannelRegistry::Get(
    const std::string& name) const {
  for (auto& ch : channels_) {
    if (ch->GetName() == name) {
      return ch.get();
    }
  }
  return nullptr;
}

std::vector<std::string>
ChannelRegistry::ListChannels() const {
  std::vector<std::string> names;
  names.reserve(channels_.size());
  for (auto& ch : channels_) {
    names.push_back(ch->GetName());
  }
  return names;
}

} // namespace tizenclaw
