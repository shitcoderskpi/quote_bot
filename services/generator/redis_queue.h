//
// Created by mmatz on 7/16/25.
//

#ifndef REDIS_QUEUE_H
#define REDIS_QUEUE_H

#include <spdlog/spdlog.h>

#include "cppcoro/static_thread_pool.hpp"
#include "cppcoro/task.hpp"
#include "exceptions.h"
#include "redis_reply.h"

class redis_queue final {
public:
    redis_queue(const std::string_view &host, int port, cppcoro::static_thread_pool &pool) noexcept;
    ~redis_queue() noexcept = default;

    template<typename StringT>
    cppcoro::task<redis_reply> enqueue(const std::string_view &queue_name, StringT data) const;

    cppcoro::task<redis_reply> enqueue(const std::string_view &queue_name, const char *data) const;

    cppcoro::task<redis_reply> dequeue(const std::string_view &queue_name, size_t timeout) const;

    [[nodiscard]] cppcoro::task<size_t> size(const std::string_view &queue_name) const;

    cppcoro::task<> delete_queue(const std::string_view &queue_name) const;

private:
    std::unique_ptr<redisContext, decltype(&redisFree)> c;
    std::shared_ptr<spdlog::logger> _logger;
    cppcoro::static_thread_pool &_pool;
};

template<typename StringT>
cppcoro::task<redis_reply> redis_queue::enqueue(const std::string_view &queue_name, StringT data) const {
    co_await _pool.schedule();

    if (!c)
        throw exceptions::redis::empty_context_error{};

    constexpr std::string_view operation = "LPUSH";
    constexpr size_t operation_size = operation.size();
    constexpr size_t argc = 3;

    if constexpr (std::is_same_v<StringT, std::string>) {
        const char *argv[] = {operation.data(), queue_name.data(), data.c_str()};
        const size_t argvlen[] = {operation_size, queue_name.size(), data.size()};
        redis_reply reply{redisCommandArgv(c.get(), argc, argv, argvlen)};
        co_return reply;
    } else if constexpr (std::is_same_v<StringT, std::string_view>) {
        const char *argv[] = {operation.data(), queue_name.data(), data.data()};
        const size_t argvlen[] = {operation_size, queue_name.size(), data.size()};
        redis_reply reply{redisCommandArgv(c.get(), argc, argv, argvlen)};
        co_return reply;
    } else if constexpr (std::is_same_v<StringT, const char *>) {
        const char *argv[] = {operation.data(), queue_name.data(), data};
        const size_t argvlen[] = {operation_size, queue_name.size(),
                                  strnlen(data, std::numeric_limits<size_t>::max() - 1)};
        redis_reply reply{redisCommandArgv(c.get(), argc, argv, argvlen)};
        co_return reply;
    } else if constexpr (std::is_same_v<StringT, char *>) {
        const char *argv[] = {operation.data(), queue_name.data(), data};
        const size_t argvlen[] = {operation_size, queue_name.size(),
                                  strnlen(data, std::numeric_limits<size_t>::max() - 1)};
        redis_reply reply{redisCommandArgv(c.get(), argc, argv, argvlen)};
        co_return reply;
    }

    throw exceptions::redis::mismatched_type_error{"Mismatched string type."};
}


#endif // REDIS_QUEUE_H
