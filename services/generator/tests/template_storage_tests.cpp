//
// Created by mmatz on 7/28/25.
//

#include <future>
#include <gtest/gtest.h>

#include "../storage.h"
#include "cppcoro/sync_wait.hpp"
#include "globals.h"

templates::storage storage {};
extern cppcoro::static_thread_pool thread_pool;

constexpr std::string_view template_path = "../templates/tg_template.svg";
constexpr std::string_view template_dir =  "../templates";

int count_files_in_directory(const std::filesystem::path &path) {
    int count = 0;
    for (const auto& entry : std::filesystem::directory_iterator(path)) {
        if (entry.is_regular_file()) {
            ++count;
        }
    }
    return count;
}



TEST(template_storage_tests, load_template_sync) {
    EXPECT_NO_THROW(storage.load_template(template_path));
    EXPECT_EQ(storage.size(), 1);
    EXPECT_NO_THROW(storage["tg_template"]);
}

TEST(template_storage_tests, load_templates_sync) {
    EXPECT_NO_THROW(storage.load_templates(template_dir));
    EXPECT_EQ(storage.size(), count_files_in_directory(template_dir));
}

TEST(template_storage_tests, load_template_async) {
    timeout_test<void>([&] {
        cppcoro::sync_wait(storage.load_template_async(template_path, thread_pool));
    }, std::chrono::milliseconds {50                });
    EXPECT_EQ(storage.size(), 1);
    EXPECT_NO_THROW(storage["tg_template"]);
}

TEST(template_storage_tests, load_templates_async) {
    timeout_test<void>([&] {
        cppcoro::sync_wait(storage.load_templates_async(template_dir, thread_pool));
    }, std::chrono::milliseconds {100});

    EXPECT_EQ(storage.size(), count_files_in_directory(template_dir));
}
