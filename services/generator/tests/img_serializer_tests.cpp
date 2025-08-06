//
// Created by mmatz on 8/6/25.
//

#include <gtest/gtest.h>

#include "../image_serializer.h"

TEST(img_serializer, deserialization_test) {
    const auto result = image_serializer::deserialize_image("../.cache/templates/tg_template.csvg");

    ASSERT_TRUE(result.has_value());

    ASSERT_EQ(result.value().text_entries.size(), 3);
    ASSERT_EQ(result.value().text_entries.at(0).color, "#000");
}