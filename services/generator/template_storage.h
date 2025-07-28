//
// Created by mmatz on 7/24/25.
//

#ifndef TEMPLATE_STORAGE_H
#define TEMPLATE_STORAGE_H
#include <filesystem>
#include <mutex>
#include <string>
#include <unordered_map>
#include <spdlog/spdlog.h>

#include "global.h"
// Prevents name conflict
#undef linux
#include "cppcoro/static_thread_pool.hpp"
#include "cppcoro/task.hpp"


class template_storage {
public:

    template_storage() noexcept {
        logger = logger_init("template_storage");
    }
    ~template_storage() = default;

    void load_template(const std::filesystem::path& path);
    void load_templates(const std::filesystem::path& path);
    size_t size() const noexcept {
        return templates.size();
    }

    // TODO: Add async file read
    //  It will not come any time soon, cppcoro's file manipulation is broken (ಥ﹏ಥ)
    // cppcoro::task<> load_template_async(const std::filesystem::path& path, const cppcoro::static_thread_pool& pool);
    // cppcoro::task<> load_templates_async(const std::filesystem::path& path, const cppcoro::static_thread_pool& pool);

    std::string operator[] (const std::string& key);

private:
    std::unordered_map<std::string, std::string> templates;
    std::mutex mtx;
    std::shared_ptr<spdlog::logger> logger;
};



#endif //TEMPLATE_STORAGE_H
