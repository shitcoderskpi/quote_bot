//
// Created by mmatz on 7/16/25.
//

#include "redis_queue.h"
#include "globals.h"
#include "cppcoro/schedule_on.hpp"

redis_queue::redis_queue(const std::string_view &host, const int port, cppcoro::static_thread_pool& pool) noexcept : _pool {pool} {
    _logger = logger_init("redis_queue");
    c = std::shared_ptr<redisContext> {redisConnect(host.data(), port)};
    if (c == nullptr || c->err) {
        if (c) {
            _logger->error("Failed to connect to redis server: {}", c->errstr);
        } else {
            _logger->error("Failed to allocate redis context object");
        }

        exit(1);
    }
}

redis_queue::~redis_queue() {
    if (c) redisFree(c.get());
}

cppcoro::task<redis_reply> redis_queue::enqueue(const std::string_view &queue_name, const char *data) const {
    co_await _pool.schedule();

    if (!c) throw exceptions::redis::empty_context_error {};

    redis_reply reply {redisCommand(c.get(), "LPUSH %s %s", queue_name.data(), data)};

    co_return std::move(reply);
}

cppcoro::task<redis_reply> redis_queue::dequeue(const std::string_view &queue_name, const size_t timeout) const {
    co_await _pool.schedule();

    if (!c) throw exceptions::redis::empty_context_error {};

    redis_reply reply {redisCommand(c.get(), "BRPOP %s %d", queue_name.data(), timeout)};

    co_return std::move(reply);
}

cppcoro::task<size_t> redis_queue::size(const std::string_view &queue_name) const {
    co_await _pool.schedule();

    if (!c) throw exceptions::redis::empty_context_error {};

    if (const redis_reply reply { redisCommand(c.get(), "LLEN %s", queue_name) }; reply.has_value()) {
        if (reply.get_type() == REDIS_REPLY_INTEGER) {
            co_return reply.get_integer();
        }
        spdlog::warn("Redis reply to LLEN was {}", reply.get_type());
    }

    co_return 0;
}

cppcoro::task<> redis_queue::delete_queue(const std::string_view &queue_name) const {
    co_await _pool.schedule();

    if (!c) throw exceptions::redis::empty_context_error {};

    redisCommand(c.get(), "DEL %s" , queue_name.data());

    co_return;
}
