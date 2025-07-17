//
// Created by mmatz on 7/16/25.
//

#include "redis_reply.h"

redis_reply::redis_reply(redisReply *reply) {
    _reply = reply;
}

redis_reply::redis_reply(void *reply) {
    _reply = static_cast<redisReply *>(reply);
}

redis_reply::~redis_reply() {
    if (_reply) {
        freeReplyObject(_reply);
    }
}

std::optional<redisReply *> redis_reply::get_reply() {
    if (_reply) {
        return _reply;
    }
    return std::nullopt;
}
