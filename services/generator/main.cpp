#include <csignal>
#include <exception>
#include <string>
#include <spdlog/spdlog.h>

#include "Magick++/Image.h"
#include "globals.h"
#include "redis_queue.h"
#include "prometheus/exposer.h"
#include "prometheus/registry.h"
#include <Magick++.h>

#include "preprocessor.h"
#include "renderer.h"
#include "cppcoro/sync_wait.hpp"
#include "storage.h"

void sig_handler(const int sig) {
    switch (sig) {
        case SIGINT:
        case SIGTERM:
            spdlog::info("Received SIGINT/SIGTERM, exiting...");
            spdlog::drop_all();
            exit(EXIT_SUCCESS);
        default:
            spdlog::info("Received signal {}, exiting...", strsignal(sig));
            spdlog::drop_all();
            exit(EXIT_SUCCESS);
    }
}

void generate(const std::string& img, const redis_queue& queue) {
    try {
        Magick::Image image;
        image.density(Magick::Point(300, 300));
        image.read(Magick::Blob(img.data(), img.length()));
        image.magick("PNG");

        Magick::Blob blob;
        image.write(&blob);
        
        spdlog::debug("Enqueuing data to results queue");
        cppcoro::sync_wait(queue.enqueue("generate:results", image_to_base64(blob)));
    } catch (const std::exception &e){
        spdlog::error("Got exception {}", e.what());
    }
}

int main()
{
    using namespace prometheus;
    constexpr auto prometheus_host = "0.0.0.0:8080";
    const std::string queue_name = "generate:jobs";

    std::signal(SIGINT, sig_handler);
    std::signal(SIGTERM, sig_handler);

    spdlog::set_level(spdlog::level::debug);
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

    // TODO: use cli args to set number of threads
    cppcoro::static_thread_pool thread_pool {4};
    templates::storage storage {};
    cppcoro::sync_wait( storage.load_templates_async("templates/", thread_pool) );

    templates::preprocessor p {};

    auto img = p.preprocess(storage["tg_template"]);

    renderer r {};

    const redis_queue queue {redis_host, 6379, thread_pool};

    while (true) {
        auto reply = cppcoro::sync_wait(queue.dequeue(queue_name, 0));
        auto result = r.render_image(img, Magick::Point {300, 300});

        result.magick("PNG");

        logger->debug("Image dimensions: x={}|y={}", result.columns(), result.rows());

        spdlog::debug("Enqueuing data to results queue");
        cppcoro::sync_wait(queue.enqueue("generate:results", image_to_base64(result)));
    }

    spdlog::drop_all();
    return EXIT_SUCCESS;
}
