//
// Created by mmatz on 7/16/25.
//

#include "redis_reply.h"

#include <algorithm>
#include <array>

redis_reply::redis_reply(redisReply *reply, const bool can_free) noexcept
    : can_free(can_free) {
  _reply = reply;
}

redis_reply::redis_reply(void *reply, const bool can_free)
    : can_free(can_free) {
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

int redis_reply::get_type() const {
  if (_reply) {
    return _reply->type;
  }

  throw empty_reply_error();
}

long long redis_reply::get_integer() const {
  if (_reply) {
    return _reply->integer;
  }

  throw empty_reply_error();
}

double redis_reply::get_dval() const {
  if (_reply) {
    return _reply->dval;
  }

  throw empty_reply_error();
}

size_t redis_reply::get_len() const {
  if (_reply) {
    return _reply->len;
  }

  throw empty_reply_error();
}

std::string redis_reply::get_str() const {
  if (_reply) {
    return std::string {_reply->str, _reply->len};
  }

  throw empty_reply_error();
}

std::array<char, 4> redis_reply::get_vtype() const {
  if (_reply) {
    std::array<char, 4> result{};
    std::copy_n(_reply->vtype, 4, result.begin());
    return result;
  }

  throw empty_reply_error();
}

redis_array redis_reply::get_array() const {
  if (_reply) {
    return redis_array{_reply->element, _reply->elements};
  }

  throw empty_reply_error();
}

bool redis_reply::has_value() const noexcept { return _reply; }
