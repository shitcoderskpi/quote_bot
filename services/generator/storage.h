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

#include "globals.h"
#include "cppcoro/static_thread_pool.hpp"
#include "cppcoro/task.hpp"

namespace templates {
    class storage {
    public:

        storage() noexcept {
            logger = logger_init("template_storage");
        }
        ~storage() = default;

        void load_template(const std::filesystem::path& path);
        void load_templates(const std::filesystem::path& path);
        size_t size() const noexcept {
            return templates.size();
        }

        // TODO: may try to get rid of all sync file loading function, and move pool reference to class' field (???)
        // P.S.: this can be good in way to use same aio iterface w/ `redis_queue` as there reference to static_thread_pool
        //       is passed to constructor
        cppcoro::task<> load_template_async(const std::filesystem::path& path, cppcoro::static_thread_pool& pool);
        cppcoro::task<> load_templates_async(const std::filesystem::path& path, cppcoro::static_thread_pool& pool);

        std::string operator[] (const std::string& key);

    private:
        std::unordered_map<std::string, std::string> templates;
        std::mutex mtx;
        std::shared_ptr<spdlog::logger> logger;
    };
}



#endif //TEMPLATE_STORAGE_H
