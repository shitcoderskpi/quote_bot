//
// Created by mmatz on 7/28/25.
//

#include <gtest/gtest.h>

#include "../template_storage.h"

template_storage storage {};

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
    EXPECT_NO_THROW(storage.load_template("../templates/tg_template.svg"));
    EXPECT_EQ(storage.size(), 1);
    EXPECT_NO_THROW(storage["tg_template"]);
}

TEST(template_storage_tests, load_templates_sync) {
    EXPECT_NO_THROW(storage.load_templates("../templates"));
    EXPECT_EQ(storage.size(), count_files_in_directory("../templates"));
}
