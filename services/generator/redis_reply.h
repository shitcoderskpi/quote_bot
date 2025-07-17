//
// Created by mmatz on 7/16/25.
//

#ifndef REDIS_REPLY_H
#define REDIS_REPLY_H
#include <hiredis/hiredis.h>
#include <optional>


class redis_reply {
    public:
    explicit redis_reply(redisReply * reply);
    explicit redis_reply(void * reply);
    ~redis_reply();

    redis_reply(const redis_reply&) = delete;
    redis_reply& operator=(const redis_reply&) = delete;

    redis_reply(redis_reply&& other) noexcept : _reply(other._reply) {
        other._reply = nullptr;
    }

    redis_reply& operator=(redis_reply&& other) noexcept {
        if (this != &other) {
            if (_reply) {
                freeReplyObject(_reply);
            }
            _reply = other._reply;
            other._reply = nullptr;
        }
        return *this;
    }

    std::optional<redisReply*> get_reply();

private:
    redisReply *_reply;
};



#endif //REDIS_REPLY_H
