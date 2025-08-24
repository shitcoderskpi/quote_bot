//
// Created by mmatz on 8/24/25.
//
#include <gtest/gtest.h>

#include "../compressor.h"
#include "../redis_queue.h"
#include "cppcoro/sync_wait.hpp"
extern const redis_queue q;

TEST(redis_compressor_tests, compress_queue_decompress_test) {
    constexpr auto data = "test data";
    const auto compressed = compressor::compress(data, 22);
    const auto decompressed = compressor::decompress(compressed);
    EXPECT_EQ(decompressed, data);
    EXPECT_NO_THROW(cppcoro::sync_wait(q.enqueue("test", compressed)));

    EXPECT_EQ(compressor::decompress(cppcoro::sync_wait(q.dequeue("test", 0)).get_array()[1].get_str()), data);
}