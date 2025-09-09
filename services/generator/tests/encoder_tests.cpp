//
// Created by mmatz on 8/24/25.
//
#include <gtest/gtest.h>

#include "../globals.h"

TEST(encoder_tests, encode_test) {
    Magick::Image image;
    image.read("../tests/what.jpg");
    EXPECT_NO_FATAL_FAILURE(image_to_base64(image));
    // const auto data = image_to_base64(image);
    // std::cout << data << std::endl;
}