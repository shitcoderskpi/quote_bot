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
cppcoro::static_thread_pool thread_pool;
redis_queue q {"localhost", 6379, thread_pool};

TEST(redis_queue, enqueue_test) {
    EXPECT_NO_THROW(
        cppcoro::sync_wait(q.enqueue("test", "Test data"))
        );

    auto reply = cppcoro::sync_wait(q.dequeue("test", 1));

    EXPECT_EQ(reply.get_reply().has_value(), true);

    EXPECT_EQ(strcmp(reply.get_reply().value()->element[1]->str, "Test data"), 0);
}

TEST(redis_queue, delete_test) {
    EXPECT_NO_THROW(cppcoro::sync_wait(q.delete_queue("test")));
}
