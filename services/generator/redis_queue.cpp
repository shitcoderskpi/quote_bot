//
// Created by mmatz on 7/16/25.
//

#include "redis_queue.h"
#include "globals.h"
#include "cppcoro/schedule_on.hpp"

redis_queue::redis_queue(const std::string &host, const int port, cppcoro::static_thread_pool& pool) noexcept : _pool {pool} {
    _logger = logger_init("redis_queue");
    c = redisConnect(host.c_str(), port);
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
    if (c) redisFree(c);
}

cppcoro::task<redis_reply> redis_queue::enqueue(const std::string &queue_name, const char *data) const {
    co_await _pool.schedule();

    redis_reply reply {redisCommand(c, "LPUSH %s %s", queue_name.c_str(), data)};

    co_return std::move(reply);
}

cppcoro::task<redis_reply> redis_queue::dequeue(const std::string &queue_name, const size_t timeout) const {
    co_await _pool.schedule();

    redis_reply reply {redisCommand(c, "BRPOP %s %d", queue_name.c_str(), timeout)};

    co_return std::move(reply);
}

cppcoro::task<size_t> redis_queue::size(const std::string &queue_name) const {
    co_await _pool.schedule();

    if (const redis_reply reply { redisCommand(c, "LLEN %s", queue_name) }; reply.has_value()) {
        if (reply.get_type().value() == REDIS_REPLY_INTEGER) {
            co_return reply.get_integer().value();
        }
        spdlog::warn("Redis reply to LLEN was {}", reply.get_type().value());
    }

    co_return 0;
}

cppcoro::task<> redis_queue::delete_queue(const std::string &queue_name) const {
    co_await _pool.schedule();

    redisCommand(c, "DEL %s" , queue_name.c_str());

    co_return;
}
