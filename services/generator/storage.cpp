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
        const auto img = _preprocessor.preprocess(buffer, "sans-serif");
        templates.emplace(template_name, img);
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

        const auto template_name = path.stem().string();
        if (templates.contains(template_name)) {
            logger->warn("Template {} is already loaded", template_name);
            co_return;
        }

        co_await pool.schedule();
        std::lock_guard lock {mtx};

        if (!path.has_extension() || path.extension() != ".csvg") {
            std::ifstream file {path};
            if (!file) {
                logger->error("Failed to open a file {}", path.filename().string());
                co_return;
            }

            std::stringstream ss;
            ss << file.rdbuf();

            const auto img = _preprocessor.preprocess(ss.str(), "sans-serif");

            templates.emplace(template_name, img);
            logger->info("Template {} is loaded successfully", template_name);
        } else {
            const auto result = image_serializer::deserialize_image(path);
            if (!result.has_value()) {
                logger->error("Failed to deserialize image");
                co_return;
            }

            templates.emplace(template_name, result.value());
            logger->info("Template {} is loaded successfully", template_name);
        }

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
            logger->debug("Loading template file {}", entry.path().string());
            co_await load_template_async(entry, pool);
        }

        co_return;
    }

    image storage::operator[](const std::string &key) {
        std::lock_guard lock {mtx};
        return templates.at(key);
    }

    void storage::save_all(const std::filesystem::path &cache_path) {
        logger->debug("Saving templates to {}", cache_path.string());
        for (const auto &[key, img] : templates) {
            const auto filename = cache_path / (key + extension);
            if (std::filesystem::exists(filename)) continue;
            image_serializer::serialize_image(img, filename);
        }
    }
}
