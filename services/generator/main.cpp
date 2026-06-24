#include <format>
#include <spdlog/spdlog.h>
#include <string>

#include "Magick++/Image.h"
#include "compressor.h"
#include "parser.h"
#include "redis_queue.h"
#include "redis_reply.h"
#include "renderer.h"
#include "response.h"
#include "globals.h"

// #include "renderer.h"

#define definitely_not_a_void_void void


static constexpr std::string_view queue_name = "generate:jobs";
static constexpr std::string_view results_queue = "generate:results";

void on_dequeue(redisAsyncContext *ctx, definitely_not_a_void_void *reply, definitely_not_a_void_void *privdata) {
    if (!privdata) {
        spdlog::critical("Private data is null.");
        return;
    }
    if (!reply) {
        spdlog::critical("Redis reply was null.");
        return;
    }
    switch (const auto r = redis_reply{reply}; *r.get_type()){
        case REDIS_REPLY_ARRAY: {
            auto compressed = r.get_array().and_then([] (const std::span<redisReply *> array) constexpr -> std::expected<std::string, std::string> {
                const auto str_ptr = array[1]->str;
                const auto len = array[1]->len;
                if (!str_ptr) {
                    return std::unexpected {"String pointer is null."};
                }
                return compressor::decompress(std::string_view {str_ptr, len});

            }).and_then([privdata](const std::string &decompressed) constexpr -> std::expected<std::string, std::string> {
                const auto result = parser::parse(decompressed);
                if (!result) {
                    spdlog::critical("Failed to parse decompressed string: {}", result.error());
                }
                const auto rend = static_cast<renderer *>(privdata);
                // 108 dpi is good enough for me. At least rendering time is down 4 times.
                auto img = rend->render_image(*result, Magick::Point{108, 108});

                img.depth(8);
                img.alpha(true);

                img.magick("WEBP");
                img.defineValue("webp", "lossless", "true");

                spdlog::debug("Image dimensions: x={}|y={}", img.columns(), img.rows());
                spdlog::debug("Using {} bits per channel, color space: {}, alpha channel: {}", img.depth(),
                              colorspace_type_to_string(img.colorSpace()), img.alpha());

#ifdef DEBUG
                img.write("output.webp");
#endif
                const auto encoded = image_to_base64(img);
                const response res = {result->metadata.chat_id, encoded};
                const auto comp = compressor::compress(res.to_json());

                if (comp.empty()) {
                    return std::unexpected {"Could not compress the data."};
                }
                return comp;
            });
            if (!compressed) {
                spdlog::critical("Failed to render image: {}", compressed.error());
                break;
            }
            redis_queue::enqueue(ctx, results_queue, *compressed, on_dequeue, privdata);
        } break;
        default: break;
    }
    redis_queue::dequeue(ctx, queue_name, 0, on_dequeue, privdata);
}

int main() {
    constexpr auto redis_port = 6379;

    spdlog::set_level(spdlog::level::debug);
    spdlog::info("Logging setup is done.");

    const auto redis_host = getenv("REDIS_HOST");
    if (!redis_host) {
        spdlog::critical("REDIS_HOST is not set in the environment.");
        exit(EXIT_FAILURE);
    }

    const unsigned int cores = std::thread::hardware_concurrency();
    spdlog::debug("Hardware concurrency: {}", cores);


    #pragma omp parallel for schedule(dynamic) num_threads(cores) default(none) shared(redis_host, redis_port, cores, queue_name)
    for (unsigned int i = 0; i < cores; ++i) {
        uv_loop_t loop;
        uv_loop_init(&loop);

        renderer r{};
        auto queue = redis_queue::create(redis_host, redis_port, &loop);
        if (!queue) {
            spdlog::critical("Failed to create redis queue: {}", queue.error());
            continue;
        }
        spdlog::info("Redis started at {}:{}", redis_host, redis_port);
        queue->dequeue<renderer>(queue_name, 0, on_dequeue, &r);
        uv_run(&loop, UV_RUN_DEFAULT);
        uv_loop_close(&loop);
    }
    return EXIT_SUCCESS;
}