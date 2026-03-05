#ifndef __CONTAINER_ENGINE_H__
#define __CONTAINER_ENGINE_H__

#include <string>
#include <memory>

namespace tizenclaw {


class ContainerEngine {
public:
    ContainerEngine();
    ~ContainerEngine();

    // Initialize the container backend (crun or runc)
    bool Initialize();

    // Setup container rootfs, generate config.json, and execute a skill command
    // capturing the JSON output synchronously from a long-running container.
    std::string ExecuteSkill(const std::string& skill_name, const std::string& arg_str);

private:
    bool EnsureSkillsContainerRunning();
    bool PrepareSkillsBundle();
    bool IsContainerRunning() const;
    bool StartSkillsContainer();
    void StopSkillsContainer();
    bool WriteSkillsConfig() const;
    std::string BuildPaths(const std::string& leaf) const;
    std::string EscapeShellArg(const std::string& input) const;

    bool m_initialized;
    std::string m_runtime_bin;
    std::string m_app_data_dir;
    std::string m_skills_dir;
    std::string m_bundle_dir;
    std::string m_rootfs_tar;
    std::string m_container_id;
};

} // namespace tizenclaw

#endif // __CONTAINER_ENGINE_H__
