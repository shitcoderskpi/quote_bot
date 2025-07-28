//
// Created by mmatz on 7/24/25.
//
#include <fstream>

#include "template_storage.h"

#include "cppcoro/read_only_file.hpp"

void template_storage::load_template(const std::filesystem::path &path) {
    if (!std::filesystem::exists(path) || !std::filesystem::is_regular_file(path)) {
        spdlog::error("Template file {} does not exist or is not a regular file", path.string());
        return;
    }

    std::lock_guard lock {mtx};

    const auto length = std::filesystem::file_size(path);
    std::ifstream file {path};
    if (!file.is_open()) {
        spdlog::error("Failed to open file {}", path.string());
        return;
    }
    const auto buffer = new char[length];
    logger->debug("Loading template file {}", path.string());
    file.read(buffer, length);

    auto template_name = path.stem().string();
    templates.emplace(template_name, std::string {buffer});
    logger->info("Template {} is loaded successfully", template_name);

    delete[] buffer;
    file.close();
}

void template_storage::load_templates(const std::filesystem::path &path) {
    if (!std::filesystem::exists(path) || !std::filesystem::is_directory(path)) {
        spdlog::error("Template directory {} does not exist, or is not a directory", path.string());
        return;
    }


    for (const auto &entry : std::filesystem::directory_iterator(path)) {
        load_template(entry.path());
    }
}

std::string template_storage::operator[](const std::string &key) {
    std::lock_guard lock {mtx};
    return templates.at(key);
}
