#include <dlog.h>
#include <curl/curl.h>
#include <fstream>
#include <iostream>
#include <dirent.h>
#include <sys/stat.h>

#include "agent_core.h"

#ifdef  LOG_TAG
#undef  LOG_TAG
#endif
#define LOG_TAG "TizenClaw_AgentCore"

AgentCore::AgentCore() : m_container(new ContainerEngine()), m_initialized(false) {
    // Constructor
}

AgentCore::~AgentCore() {
    Shutdown();
}

bool AgentCore::Initialize() {
    if (m_initialized) return true;

    dlog_print(DLOG_INFO, LOG_TAG, "AgentCore Initializing...");
    
    if (!m_container->Initialize()) {
        dlog_print(DLOG_ERROR, LOG_TAG, "Failed to initialize LXC Container Engine");
        return false;
    }

    std::ifstream key_file("/usr/apps/org.tizen.tizenclaw/data/gemini_api_key.txt");
    if (key_file.is_open()) {
        std::getline(key_file, m_gemini_api_key);
        dlog_print(DLOG_INFO, LOG_TAG, "Loaded Gemini API Key (Length: %zu)", m_gemini_api_key.length());
        key_file.close();
    } else {
        dlog_print(DLOG_ERROR, LOG_TAG, "Gemini API key file not found: /usr/apps/org.tizen.tizenclaw/data/gemini_api_key.txt");
    }

    curl_global_init(CURL_GLOBAL_DEFAULT);

    m_initialized = true;
    return true;
}

void AgentCore::Shutdown() {
    if (!m_initialized) return;

    dlog_print(DLOG_INFO, LOG_TAG, "AgentCore Shutting down...");
    
    m_container.reset();
    curl_global_cleanup();
    
    m_initialized = false;
}

void AgentCore::ProcessPrompt(const std::string& prompt) {
    if (!m_initialized) {
        dlog_print(DLOG_ERROR, LOG_TAG, "Cannot process prompt. AgentCore not initialized.");
        return;
    }

    dlog_print(DLOG_INFO, LOG_TAG, "AgentCore received prompt: %s", prompt.c_str());

    std::string gemini_response = QueryGemini(prompt);
    
    try {
        auto json_res = nlohmann::json::parse(gemini_response);
        if (json_res.contains("candidates") && !json_res["candidates"].empty()) {
            auto parts = json_res["candidates"][0]["content"]["parts"];
            for (auto& part : parts) {
                if (part.contains("functionCall")) {
                    std::string skill_name = part["functionCall"]["name"];
                    auto args = part["functionCall"]["args"];
                    dlog_print(DLOG_INFO, LOG_TAG, "Gemini requested function: %s", skill_name.c_str());
                    ExecuteSkill(skill_name, args);
                } else if (part.contains("text")) {
                    std::string text_reply = part["text"];
                    dlog_print(DLOG_INFO, LOG_TAG, "Gemini Text Reply: %s", text_reply.c_str());
                }
            }
        } else {
            dlog_print(DLOG_ERROR, LOG_TAG, "Gemini API returned an error or empty candidates.");
        }
    } catch (const std::exception& e) {
        dlog_print(DLOG_ERROR, LOG_TAG, "Failed to parse Gemini JSON: %s", e.what());
    }
}

static size_t WriteCallback(void *contents, size_t size, size_t nmemb, void *userp) {
    ((std::string*)userp)->append((char*)contents, size * nmemb);
    return size * nmemb;
}

std::string AgentCore::QueryGemini(const std::string& prompt_text) {
    if (m_gemini_api_key.empty()) {
        dlog_print(DLOG_ERROR, LOG_TAG, "API Key is empty! Please create /usr/apps/org.tizen.tizenclaw/data/gemini_api_key.txt");
        return "{}";
    }
    
    std::vector<nlohmann::json> dynamic_functions;
    const std::string skills_dir = "/usr/apps/org.tizen.tizenclaw/data/skills";
    DIR *dir = opendir(skills_dir.c_str());
    if (dir) {
        struct dirent *ent;
        while ((ent = readdir(dir)) != NULL) {
            if (ent->d_name[0] == '.') continue;
            std::string manifest_path = skills_dir + "/" + ent->d_name + "/manifest.json";
            std::ifstream mf(manifest_path);
            if (mf.is_open()) {
                try {
                    nlohmann::json j;
                    mf >> j;
                    if (j.contains("parameters")) {
                        nlohmann::json f;
                        f["name"] = j.value("name", ent->d_name);
                        f["description"] = j.value("description", "");
                        f["parameters"] = j["parameters"];
                        dynamic_functions.push_back(f);
                    }
                } catch (...) {
                    dlog_print(DLOG_WARN, LOG_TAG, "Failed to parse manifest: %s", manifest_path.c_str());
                }
            }
        }
        closedir(dir);
    }

    nlohmann::json payload = {
        {"contents", {{
            {"parts", {{{"text", prompt_text}}}}
        }}}
    };

    if (!dynamic_functions.empty()) {
        payload["tools"] = {{
            {"functionDeclarations", dynamic_functions}
        }};
    }

    
    std::string url = "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key=" + m_gemini_api_key;
    std::string response_string;
    
    CURL *curl = curl_easy_init();
    if (curl) {
        curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
        struct curl_slist *headers = NULL;
        headers = curl_slist_append(headers, "Content-Type: application/json");
        curl_easy_setopt(curl, CURLOPT_HTTPHEADER, headers);

        std::string json_str = payload.dump();
        curl_easy_setopt(curl, CURLOPT_POSTFIELDS, json_str.c_str());
        
        curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, WriteCallback);
        curl_easy_setopt(curl, CURLOPT_WRITEDATA, &response_string);
        curl_easy_setopt(curl, CURLOPT_SSL_VERIFYPEER, 0L); // Bypass for emulator testing

        CURLcode res = curl_easy_perform(curl);
        if (res != CURLE_OK) {
            dlog_print(DLOG_ERROR, LOG_TAG, "curl_easy_perform() failed: %s", curl_easy_strerror(res));
        }
        curl_slist_free_all(headers);
        curl_easy_cleanup(curl);
    }
    return response_string;
}

bool AgentCore::ExecuteSkill(const std::string& skill_name, const nlohmann::json& args) {
    dlog_print(DLOG_INFO, LOG_TAG, "Executing skill logic: %s", skill_name.c_str());
    
    // Launching the predefined container environment for Skills execution
    m_container->StartContainer("tizenclaw_skill_vm", "/usr/apps/org.tizen.tizenclaw/data/rootfs.tar.gz");
    
    // Convert JSON explicitly to a shell-escaped string or just write to temp file
    std::string arg_str = args.dump();
    // Escape single quotes just in case
    size_t pos = 0;
    while ((pos = arg_str.find("'", pos)) != std::string::npos) {
        arg_str.replace(pos, 1, "'\"'\"'");
        pos += 5;
    }
    
    std::string skill_file = "/usr/apps/org.tizen.tizenclaw/data/skills/" + skill_name + "/" + skill_name + ".py";
    // Using environment variable to pass JSON args generically, avoids quoting issues
    std::string cmd = "CLAW_ARGS='" + arg_str + "' python3 " + skill_file;
    
    dlog_print(DLOG_INFO, LOG_TAG, "Running Skill CMD: %s", cmd.c_str());
    int res = std::system(cmd.c_str());
    
    if (res != 0) {
        dlog_print(DLOG_ERROR, LOG_TAG, "Skill execution failed with code %d", res);
        return false;
    }
    return true;
}
