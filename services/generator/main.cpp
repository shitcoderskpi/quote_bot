#include <spdlog/spdlog.h>
#include <spdlog/async.h>
#include <spdlog/sinks/stdout_color_sinks.h>
#include <hiredis.h>
#include "prometheus/exposer.h"
#include "prometheus/registry.h"

#define PATTERN "[%Y-%m-%d %H:%M:%S,%e %l] %n: %v"

std::shared_ptr<spdlog::logger> logger_init(const std::string& name, const spdlog::level::level_enum lvl = spdlog::level::debug) {
    const auto logger = spdlog::stdout_color_mt(name);
    logger->set_level(lvl);
    logger->set_pattern(PATTERN);
    return logger;
}



[[noreturn]] int main()
{
    using namespace prometheus;
    constexpr auto prometheus_host = "0.0.0.0:8080";
    const std::string queue_name = "generate:jobs";

    const auto logger = logger_init("main");
    logger->info("Logging setup is done.");

    Exposer exposer {prometheus_host};
    logger->debug("Exposed prometheus on {}", prometheus_host);

    // metrics registry
    auto registry = std::make_shared<Registry>();

    exposer.RegisterCollectable(registry);

    const auto redis_host = getenv("REDIS_HOST");
    if (redis_host == nullptr) {
        logger->critical("REDIS_HOST is not set in the environment.");
        exit(EXIT_FAILURE);
    }

    logger->debug("Connecting to redis host: {}:6379", redis_host);
    // TODO: rewrite to async
    redisContext *c = redisConnect(redis_host, 6379);
    if (c == nullptr || c->err) {
        if (c) {
            logger->error("Error calling redisConnect(): {}", c->errstr);
            redisFree(c);
        } else {
            logger->error("Unable to allocate redis context");
        }
    }

    while (true) {
        auto *reply = static_cast<redisReply *>(redisCommand(c, "BRPOP %s %d", queue_name.c_str(), 0));

        if (reply == nullptr) {
            logger->error("Command error: {}", c->errstr);
            freeReplyObject(reply);
            redisFree(c);
            exit(1);
        }

        if (reply->type == REDIS_REPLY_STRING) {
            logger->debug("Got reply from queue: {}", reply->str);
        }
        if (reply->type == REDIS_REPLY_ARRAY) {
            logger->debug("Got {} elements: ", reply->elements);
            for (int i = 0; i < reply->elements; ++i) {
                logger->debug("{}: {}", i, reply->element[i]->str);
            }
        }
        freeReplyObject(reply);

    }

    spdlog::drop_all();
    redisFree(c);
    return EXIT_SUCCESS;
}
