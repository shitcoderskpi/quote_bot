//
// Created by mmatz on 7/16/25.
//

#ifndef REDIS_REPLY_H
#define REDIS_REPLY_H
#include <expected>
#include <hiredis/hiredis.h>
#include <memory>
#include <span>
#include <string>

class redis_array;

// VIEW!!!
class redis_reply {
public:
    explicit constexpr redis_reply(redisReply *reply) noexcept;
    explicit constexpr redis_reply(void *reply) noexcept;
    [[nodiscard]] constexpr std::expected<int, std::string> get_type() const;
    [[nodiscard]] constexpr std::expected<long long, std::string> get_integer() const;
    [[nodiscard]] constexpr std::expected<double, std::string> get_dval() const;
    [[nodiscard]] constexpr std::expected<std::size_t, std::string> get_len() const;
    [[nodiscard]] constexpr std::expected<std::string_view, std::string> get_str() const;
    [[nodiscard]] constexpr std::expected<std::span<char, 4>, std::string> get_vtype() const;
    [[nodiscard]] constexpr std::expected<std::span<redisReply*>, std::string> get_array() const;
    [[nodiscard]] constexpr bool has_value() const noexcept;

private:
    redisReply *_reply;
};
constexpr redis_reply::redis_reply(redisReply *reply) noexcept
: _reply(reply) {}
constexpr redis_reply::redis_reply(void *reply) noexcept
: _reply(static_cast<redisReply *>(reply)) {}
constexpr std::expected<int, std::string> redis_reply::get_type() const {
    if (!_reply) {
        return std::unexpected {"Redis reply is null."};
    }

    return _reply->type;
}
constexpr std::expected<long long, std::string> redis_reply::get_integer() const {
    if (!_reply) {
        return std::unexpected {"Redis reply is null."};
    }

    return _reply->integer;
}
constexpr std::expected<double, std::string> redis_reply::get_dval() const {
    if (!_reply) {
        return std::unexpected {"Redis reply is null."};
    }

    return _reply->dval;
}
constexpr std::expected<std::size_t, std::string> redis_reply::get_len() const {
    if (!_reply) {
        return std::unexpected {"Redis reply is null."};
    }

    return _reply->len;
}
constexpr std::expected<std::string_view, std::string> redis_reply::get_str() const {
    if (!_reply) {
        return std::unexpected {"Redis reply is null."};
    }
    return _reply->str;
}
constexpr std::expected<std::span<char, 4>, std::string> redis_reply::get_vtype() const {
    if (!_reply) {
        return std::unexpected {"Redis reply is null."};
    }

    return std::span {_reply->vtype};
}
constexpr std::expected<std::span<redisReply*>, std::string> redis_reply::get_array() const {
    if (!_reply) {
        return std::unexpected {"Redis reply is null."};
    }

    return std::span {_reply->element, _reply->len};
}
constexpr bool redis_reply::has_value() const noexcept {
    return static_cast<bool>(_reply);
}

#endif // REDIS_REPLY_H
