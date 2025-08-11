#include <csignal>
#include <iostream>
#include <string>
#include <spdlog/spdlog.h>

#include "Magick++/Image.h"
#include "redis_queue.h"
#include <Magick++.h>

#include "compressor.h"
#include "json_parser.h"
#include "renderer.h"
#include "response.h"
#include "cppcoro/sync_wait.hpp"
#include "storage.h"

int main()
{
    const std::string queue_name = "generate:jobs";

    std::signal(SIGINT, sig_handler);
    std::signal(SIGTERM, sig_handler);

    spdlog::set_level(spdlog::level::debug);
    const auto logger = logger_init("main");
    logger->info("Logging setup is done.");

    const auto redis_host = getenv("REDIS_HOST");
    if (redis_host == nullptr) {
        logger->critical("REDIS_HOST is not set in the environment.");
        exit(EXIT_FAILURE);
    }

    const unsigned int cores = std::thread::hardware_concurrency();

    logger->debug("Hardware concurrency: {}", cores);

    cppcoro::static_thread_pool thread_pool {cores};

    const renderer r {};
    const redis_queue queue {redis_host, 6379, thread_pool};
    logger->info("Redis started at {}:{}", redis_host, 6379);

    const auto preprocessor = templates::preprocessor {};
    auto parser = request_json_parser {};
    while (!shutdown) {
        //==================================================================//
        //!!!!!!!!!!!!!!!!!!!!!!!!! IMPORTANT NOTE !!!!!!!!!!!!!!!!!!!!!!!!!//
        //------------------------------------------------------------------//
        // >>> Now assuming that reply looks like this:                     //
        //      "queue_name", "zstd compressed data (json)"                 //
        //------------------------------------------------------------------//
        // >>> After decompressing:                                         //
        //      "queue_name", "{                                            //
        //              "chat_id": id,                                      //
        //              "template": "svg data"                              //
        //      }"                                                          //
        //==================================================================//

        if (auto reply = cppcoro::sync_wait(queue.dequeue(queue_name, 0)); reply.has_value()) {
            if (reply.get_type().value() == REDIS_REPLY_ERROR) {
                const auto err_str = reply.get_str().value();
                logger->error("Redis error: {}", err_str);
                if (err_str.starts_with("MISCONF Valkey is configured to save RDB snapshots")) {
                    logger->error("Try restarting redis server.");
                }
                throw std::runtime_error("");
            }

            const auto decompressed = compressor::decompress(reply.get_array().value()[1].get_str().value());
            if (decompressed.empty()) {
                logger->error("Could not decompress given data: {}.", reply.get_array().value()[1].get_str().value());
                continue;
            }

            const auto [chat_id, svg_template] = parser.parse(decompressed);
            const auto img = preprocessor.preprocess(svg_template, "sans-serif");
            auto result = r.render_image(img, Magick::Point {300, 300});

            result.quantizeColorSpace(Magick::ColorspaceType::DisplayP3Colorspace);
            result.depth(8);
            result.alpha(true);

            result.magick("WEBP");

            logger->debug("Image dimensions: x={}|y={}", result.columns(), result.rows());
            logger->debug("Using {} bits per channel, color space: {}, alpha channel: {}", result.depth(), colorspace_type_to_string(result.colorSpace()), result.alpha());

            logger->debug("Enqueuing data to results queue");

            response res = {chat_id, image_to_base64(result)};

            const auto compressed = compressor::compress(res.to_json());
            if (compressed.empty()) {
                logger->error("Could not compress the data.");
                continue;
            }

            cppcoro::sync_wait(queue.enqueue("generate:results", compressed));
        } else {
            logger->error("Bad data was received");
        }
    }

    spdlog::drop_all();
    return EXIT_SUCCESS;
}
