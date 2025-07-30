//
// Created by mmatz on 7/28/25.
//
#include <gtest/gtest.h>

#include "../redis_queue.h"
#include "cppcoro/static_thread_pool.hpp"
#include "cppcoro/sync_wait.hpp"

///### REDIS QUEUE TESTS ###
///
///!!! TO RUN TESTS YOU NEED FIRST START REDIS (VALKEY) SERVER !!!
///
cppcoro::static_thread_pool thread_pool {4};
redis_queue q {"localhost", 6379, thread_pool};

TEST(redis_queue, enqueue_test) {
    EXPECT_NO_THROW(
        cppcoro::sync_wait(q.enqueue("test", "Test data"))
        );

    const auto reply = cppcoro::sync_wait(q.dequeue("test", 1));

    EXPECT_EQ(reply.has_value(), true);

    EXPECT_EQ(strcmp(reply.get_array().value()[1].get_str().value().c_str(), "Test data"), 0);
}

TEST(redis_array, iterator_test) {
    cppcoro::sync_wait(q.enqueue("test", "Test data"));

    const auto reply = cppcoro::sync_wait(q.dequeue("test", 1));

    EXPECT_EQ(reply.has_value(), true);

    EXPECT_EQ(reply.get_array().value().size(), 2);
    EXPECT_NO_FATAL_FAILURE(reply.get_array().value().begin());
    EXPECT_NO_FATAL_FAILURE(reply.get_array().value().end());
}

TEST(redis_queue, delete_test) {
    EXPECT_NO_THROW(cppcoro::sync_wait(q.delete_queue("test")));
}

// TEST(redis_queue, enqueue_stress_test) {
//     constexpr size_t N = 1'000;
//     const std::chrono::high_resolution_clock::time_point begin = std::chrono::high_resolution_clock::now();
//     for (size_t i {}; i < N; ++i) {
//         cppcoro::sync_wait(q.enqueue("test", "Test data"));
//     }
//
//     const std::chrono::high_resolution_clock::time_point end = std::chrono::high_resolution_clock::now();
//     const auto diff = std::chrono::duration_cast<std::chrono::microseconds>(end - begin);
//
//     std::cout << "Total time: " << diff << std::endl;
//     std::cout << "For one operation: " << diff / N << std::endl;
// }
