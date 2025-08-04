//
// Created by mmatz on 7/24/25.
//
#include <filesystem>
#include <fstream>
#include <mutex>
#include <sstream>

#include "storage.h"
namespace templates {
    void storage::load_template(const std::filesystem::path &path) {
        if (!std::filesystem::exists(path) || !std::filesystem::is_regular_file(path)) {
            logger->error("Template file {} does not exist or is not a regular file", path.string());
            return;
        }

        std::lock_guard lock {mtx};

        const auto length = std::filesystem::file_size(path);
        std::ifstream file {path};
        if (!file.is_open()) {
            logger->error("Failed to open file {}", path.string());
            return;
        }
        const auto buffer = new char[length];
        logger->debug("Loading template file {}", path.string());
        file.read(buffer, length);

        const auto template_name = path.stem().string();
        templates.emplace(template_name, std::string {buffer});
        logger->info("Template {} is loaded successfully", template_name);

        delete[] buffer;
        file.close();
    }

    void storage::load_templates(const std::filesystem::path &path) {
        if (!std::filesystem::exists(path) || !std::filesystem::is_directory(path)) {
            logger->error("Template directory {} does not exist, or is not a directory", path.string());
            return;
        }

        for (const auto &entry : std::filesystem::directory_iterator(path)) {
            load_template(entry.path());
        }
    }

    cppcoro::task<> storage::load_template_async(const std::filesystem::path &path,
        cppcoro::static_thread_pool &pool) {
        if (!std::filesystem::exists(path) || !std::filesystem::is_regular_file(path)){
            logger->error("Template file {} does not exist or is not a regular file", path.string());
            co_return;
        }

        co_await pool.schedule();
        std::lock_guard lock {mtx};

        std::ifstream file {path};
        if (!file) {
            logger->error("Failed to open a file {}", path.filename().string());
            co_return;
        }

        std::stringstream ss;
        ss << file.rdbuf();

        const auto template_name = path.stem().string();
        templates.emplace(template_name, ss.str());
        logger->info("Template {} is loaded successfully", template_name);

        co_return;
    }

    cppcoro::task<> storage::load_templates_async(const std::filesystem::path &path,
                                                                cppcoro::static_thread_pool &pool) {
        if (!std::filesystem::exists(path) || !std::filesystem::is_directory(path)) {
            logger->error("Template directory {} does not exist, or is not a directory", path.string());
            co_return;
        }

        co_await pool.schedule();

        for (const auto &entry: std::filesystem::directory_iterator(path)){
            co_await load_template_async(entry, pool);
        }

        co_return;
    }

    std::string storage::operator[](const std::string &key) {
        std::lock_guard lock {mtx};
        return templates.at(key);
    }
}
