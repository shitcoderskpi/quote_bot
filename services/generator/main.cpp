#include <format>
#include <spdlog/spdlog.h>
#include <stdexcept>
#include <string>

#include <Magick++.h>
#include "Magick++/Image.h"
#include "preprocessor.h"
#include "redis_queue.h"

#include "compressor.h"
#include "cppcoro/sync_wait.hpp"
#include "json_parser.h"
#include "prometheus/counter.h"
#include "prometheus/exposer.h"
#include "prometheus/registry.h"
#include "renderer.h"
#include "response.h"

[[noreturn]] int main() {
    const std::string queue_name = "generate:jobs";
    constexpr auto prometheus_host = "0.0.0.0:8080";
    constexpr auto redis_port = 6379;

    spdlog::set_level(spdlog::level::debug);
    const auto logger = logger_init("main");
    logger->info("Logging setup is done.");

    const auto redis_host = getenv("REDIS_HOST");
    if (redis_host == nullptr) {
        logger->critical("REDIS_HOST is not set in the environment.");
        exit(EXIT_FAILURE);
    }

    auto registry = std::make_shared<prometheus::Registry>();

    // NOTE:
    // Without patch segfaults:
    prometheus::Exposer exposer{prometheus_host};

    auto &requests_counter = prometheus::BuildCounter()
                                     .Name("redis_requests_total")
                                     .Help("Total number of Redis requests handled")
                                     .Register(*registry)
                                     .Add({{"operation", "BRPOP"}});

    exposer.RegisterCollectable(registry);

    const unsigned int cores = std::thread::hardware_concurrency();
    logger->debug("Hardware concurrency: {}", cores);

    cppcoro::static_thread_pool thread_pool{cores};

    const renderer r{};
    const redis_queue queue{redis_host, redis_port, thread_pool};
    logger->info("Redis started at {}:{}", redis_host, redis_port);

    auto parser = request_json_parser{};
    while (true) {
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
            if (reply.get_type() == REDIS_REPLY_ERROR) {
                const auto err_str = reply.get_str();
                logger->error("Redis error: {}", err_str);
                if (err_str.starts_with("MISCONF Valkey is configured to save RDB snapshots")) {
                    logger->error("Try restarting redis server.");
                }
                throw std::runtime_error("");
            }

            try {
                requests_counter.Increment();
                const auto decompressed = compressor::decompress(reply.get_array()[1].get_str());

                if (decompressed.empty()) {
                    throw std::runtime_error("Could not decompress given data.");
                }

                const auto [chat_id, svg_template] = parser.parse(decompressed);
                const auto img = templates::preprocessor::preprocess(svg_template, "sans-serif");
                auto result = r.render_image(img, Magick::Point{300, 300});

                // NOTE: WEBP does not support DCI/P3
                // result.quantizeColorSpace(Magick::ColorspaceType::DisplayP3Colorspace);
                result.depth(8);
                result.alpha(true);

                result.magick("WEBP");
                result.defineValue("webp", "lossless", "true");

                logger->debug("Image dimensions: x={}|y={}", result.columns(), result.rows());
                logger->debug("Using {} bits per channel, color space: {}, alpha channel: {}", result.depth(),
                              colorspace_type_to_string(result.colorSpace()), result.alpha());

#ifdef DEBUG
                result.write("output.webp");
#endif

                logger->debug("Enqueuing data to results queue");

                const auto encoded = image_to_base64(result);
                response res = {chat_id, encoded};

                const auto compressed = compressor::compress(res.to_json());
                if (compressed.empty()) {
                    throw std::runtime_error("Could not compress the data.");
                }

                cppcoro::sync_wait(queue.enqueue("generate:results", compressed));
            } catch (const simdjson::simdjson_error &e){
                logger->error("JSON parsing error: {}", e.what());
            } catch (const std::exception &e) {
                logger->error(e.what());
            }
        } else {
            logger->error("Bad data was received");
        }
    }
}
