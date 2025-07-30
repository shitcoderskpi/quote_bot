//
// Created by mmatz on 7/16/25.
//

#include "redis_reply.h"

#include <algorithm>
#include <array>

redis_reply::redis_reply(redisReply *reply, const bool can_free) noexcept : can_free(can_free) {
    _reply = reply;
}

redis_reply::redis_reply(void *reply, const bool can_free): can_free(can_free) {
    _reply = static_cast<redisReply *>(reply);
}

redis_reply::~redis_reply() {
    if (_reply && can_free) {
        freeReplyObject(_reply);
    }
}

std::optional<redisReply *> redis_reply::get_reply() const noexcept {
    if (_reply) {
        return _reply;
    }

    return std::nullopt;
}

std::optional<int> redis_reply::get_type() const noexcept {
    if (_reply) {
        return _reply->type;
    }

    return std::nullopt;
}

std::optional<long long> redis_reply::get_integer() const noexcept {
    if (_reply) {
        return _reply->integer;
    }

    return std::nullopt;
}

std::optional<double> redis_reply::get_dval() const noexcept {
    if (_reply) {
        return _reply->dval;
    }

    return std::nullopt;
}

std::optional<size_t> redis_reply::get_len() const noexcept {
    if (_reply) {
        return _reply->len;
    }

    return std::nullopt;
}

std::optional<std::string> redis_reply::get_str() const noexcept {
    if (_reply) {
        return _reply->str;
    }

    return std::nullopt;
}

std::optional<std::array<char, 4>> redis_reply::get_vtype() const noexcept {
    if (_reply) {
        std::array<char, 4> result {};
        std::copy_n(_reply->vtype, 4, result.begin());
        return result;
    }

    return std::nullopt;
}

std::optional<redis_array> redis_reply::get_array() const noexcept {
    if (_reply) {
        return redis_array {_reply->element, _reply->elements};
    }

    return std::nullopt;
}

bool redis_reply::has_value() const noexcept {
    return _reply != nullptr;
}
