//
// Created by mmatz on 7/16/25.
//

#ifndef REDIS_QUEUE_H
#define REDIS_QUEUE_H

#include <hiredis/async.h>
#include <hiredis/adapters/libuv.h>
#include <expected>
#include <spdlog/spdlog.h>


class redis_queue final {
public:
    constexpr static std::expected<redis_queue, std::string> create(std::string_view host, int port, uv_loop_t *loop);

    template <typename PrivData>
    constexpr void enqueue(std::string_view queue_name, std::string &body, redisCallbackFn *callback, PrivData *data = nullptr) const noexcept;
    template <typename PrivData>
    constexpr void dequeue(std::string_view queue_name, size_t timeout, redisCallbackFn *callback, PrivData *data = nullptr) const noexcept;

    template <typename PrivData>
    constexpr static void enqueue(redisAsyncContext *ctx, std::string_view queue_name, std::string &body, redisCallbackFn *callback, PrivData *data = nullptr) noexcept;
    template <typename PrivData>
    constexpr static void dequeue(redisAsyncContext *ctx, std::string_view queue_name, size_t timeout, redisCallbackFn *callback, PrivData *data = nullptr) noexcept;

private:
    redisAsyncContext *ctx;

    explicit redis_queue(redisAsyncContext *ctx) noexcept
    : ctx(ctx) {}
};

constexpr std::expected<redis_queue, std::string> redis_queue::create(const std::string_view host, const int port, uv_loop_t *loop) {
    if (loop == nullptr) {
        return std::unexpected {"libuv pointer is null."};
    }
    const auto ctx = redisAsyncConnect(host.data(), port);
    if (ctx == nullptr || ctx->err) {
        return std::unexpected {std::format("Failed to connect to redis server: {}", ctx->errstr == nullptr ? "Unknown error." : ctx->errstr)};
    }

    redisLibuvAttach(ctx, loop);
    return redis_queue {ctx};
}
template <typename PrivData>
constexpr void redis_queue::enqueue(const std::string_view queue_name, std::string &body,
                                    redisCallbackFn *callback, PrivData *data) const noexcept {
    redisAsyncCommand(ctx, callback, data, "LPUSH %s %b", queue_name.data(), body.data(), body.size());
}
template <typename PrivData>
constexpr void redis_queue::dequeue(const std::string_view queue_name, const size_t timeout,
                                    redisCallbackFn *callback, PrivData *data) const noexcept {
    redisAsyncCommand(ctx, callback, data, "BRPOP %s %d", queue_name.data(), timeout);
}
template<typename PrivData>
constexpr void redis_queue::enqueue(redisAsyncContext *ctx, const std::string_view queue_name, std::string &body,
                                    redisCallbackFn *callback, PrivData *data) noexcept {
    redisAsyncCommand(ctx, callback, data, "LPUSH %s %b", queue_name.data(), body.data(), body.size());
}
template<typename PrivData>
constexpr void redis_queue::dequeue(redisAsyncContext *ctx, const std::string_view queue_name, const size_t timeout,
                                    redisCallbackFn *callback, PrivData *data) noexcept {
    redisAsyncCommand(ctx, callback, data, "BRPOP %s %d", queue_name.data(), timeout);
}

#endif // REDIS_QUEUE_H
