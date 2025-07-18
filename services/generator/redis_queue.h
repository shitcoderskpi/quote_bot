//
// Created by mmatz on 7/16/25.
//

#ifndef REDIS_QUEUE_H
#define REDIS_QUEUE_H

#include <async.h>
#include <spdlog/spdlog.h>

#include "redis_reply.h"
#include "cppcoro/static_thread_pool.hpp"
#include "cppcoro/task.hpp"

template<typename T>
concept HasToString = requires(const T& t) {
    { std::to_string(t) } -> std::convertible_to<std::string>;
};

template<typename T>
concept HasToWString = requires(const T& t) {
    { std::to_wstring(t) } -> std::convertible_to<std::wstring>;
};

template<typename T>
concept IsString = std::same_as<T, std::string> || std::same_as<T, std::wstring>;

template<typename T>
concept EncodableT = HasToString<T> || HasToWString<T> || IsString<T>;

class redis_queue {
public:
    redis_queue(const std::string& host, int port, cppcoro::static_thread_pool& pool) noexcept;
    ~redis_queue();

    template<EncodableT T>
    cppcoro::task<redis_reply> enqueue(const std::string &queue_name, T data) const;

    cppcoro::task<redis_reply> enqueue(const std::string &queue_name, const char *data) const;

    cppcoro::task<redis_reply> dequeue(const std::string &queue_name, size_t timeout) const;

    [[nodiscard]] cppcoro::task<size_t> size(const std::string& queue_name) const;

    cppcoro::task<> delete_queue(const std::string &queue_name) const;
private:
    redisContext *c;
    std::shared_ptr<spdlog::logger> _logger;
    cppcoro::static_thread_pool& _pool;
};

template<EncodableT T>
cppcoro::task<redis_reply> redis_queue::enqueue(const std::string &queue_name, T data) const {
    co_await _pool.schedule();

    if constexpr (IsString<T>) {
        const char* argv[] = { "LPUSH", queue_name.c_str(), data.c_str() };
        const size_t argvlen[] = { 5, queue_name.size(), data.size() };
        redis_reply reply { redisCommandArgv(c, 3, argv, argvlen) };
        co_return reply;
    } else if constexpr (HasToString<T>) {
        if constexpr (HasToWString<T>) {
            redis_reply reply { redisCommand(c, "LPUSH %s %s", queue_name, std::to_wstring(data).c_str()) };
            co_return reply;
        } else {
            redis_reply reply { redisCommand(c, "LPUSH %s %s", queue_name, std::to_string(data).c_str()) };
            co_return reply;
        }
    }
}


#endif //REDIS_QUEUE_H
