#ifndef TIZENCLAW_INFRA_CONTAINER_ENGINE_H_
#define TIZENCLAW_INFRA_CONTAINER_ENGINE_H_

#include <string>
#include <memory>

namespace tizenclaw {

class ContainerEngine {
public:
    ContainerEngine();
    ~ContainerEngine();

    // Initialize the container backend (crun or runc)
    bool Initialize();

    // Execute a skill: tries UDS socket first, then
    // crun exec fallback, then host-direct fallback.
    std::string ExecuteSkill(
        const std::string& skill_name,
        const std::string& arg_str);

    // Execute arbitrary Python code via the skill
    // executor's execute_code command.
    std::string ExecuteCode(
        const std::string& code);

    // Execute file operations via the skill
    // executor's file_manager command.
    std::string ExecuteFileOp(
        const std::string& operation,
        const std::string& path,
        const std::string& content);

private:
    // Execute skill via Unix Domain Socket to the
    // skill_executor running in the secure container.
    std::string ExecuteSkillViaSocket(
        const std::string& skill_name,
        const std::string& arg_str);

    // Legacy: exec into running OCI container
    std::string ExecuteSkillViaCrun(
        const std::string& skill_name,
        const std::string& arg_str);

    bool EnsureSkillsContainerRunning();
    bool PrepareSkillsBundle();
    bool IsContainerRunning() const;
    bool StartSkillsContainer();
    void StopSkillsContainer();
    bool WriteSkillsConfig() const;
    std::string BuildPaths(
        const std::string& leaf) const;
    std::string EscapeShellArg(
        const std::string& input) const;
    std::string CrunCmd(
        const std::string& subcmd) const;

    // Extract last JSON-like line from raw output
    static std::string ExtractJsonResult(
        const std::string& raw);

    bool initialized_;
    std::string runtime_bin_;
    std::string app_data_dir_;
    std::string skills_dir_;
    std::string bundle_dir_;
    std::string rootfs_tar_;
    std::string container_id_;
    std::string crun_root_;

    static constexpr const char* kSkillSocketPath =
        "/tmp/tizenclaw_skill.sock";
};

} // namespace tizenclaw

#endif // TIZENCLAW_INFRA_CONTAINER_ENGINE_H_
