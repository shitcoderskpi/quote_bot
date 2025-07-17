
#include <csignal>
#include <string>
#include <spdlog/spdlog.h>

#include "global.h"
#include "redis_queue.h"
#include "prometheus/exposer.h"
#include "prometheus/registry.h"
#include <Magick++.h>

#include "cppcoro/sync_wait.hpp"
#include "extern/prometheus-cpp/util/include/prometheus/detail/base64.h"

void sig_handler(const int sig) {
    switch (sig) {
        case SIGINT:
        case SIGTERM:
            spdlog::info("Received SIGINT/SIGTERN, exiting...");
            spdlog::drop_all();
            exit(EXIT_SUCCESS);
        default:
            spdlog::info("Received signal {}", sig);
            return;
    }
}

void generate(const std::string& svg, const redis_queue& queue) {
    try {
        Magick::Image image;
        image.read(Magick::Blob(svg.data(), svg.size()));
        image.density(Magick::Point(300, 300));
        image.magick("PNG");
        Magick::Blob blob;
        image.write(&blob);
        image.write("output.png");

        const std::string base64 = image_to_base64(blob);

        spdlog::debug("Enqueuing base64 data to queue");
        cppcoro::sync_wait(queue.enqueue("generate:results", base64));
    } catch (const std::exception& e) {
        spdlog::error("Unhandled exception {}", e.what());
    }
}

int main()
{
    using namespace prometheus;
    constexpr auto prometheus_host = "0.0.0.0:1488";
    const std::string queue_name = "generate:jobs";

    std::signal(SIGINT, sig_handler);
    std::signal(SIGTERM, sig_handler);

    const std::string test_svg = "<svg width='800' height='800' xmlns='http://www.w3.org/2000/svg'>"
        "  <rect x='80' y='80' width='640' height='640' fill='#f0f'/>"
        "  <text x='400' y='400' font-size='48' text-anchor='middle' fill='white'>"
        "    Emoji: 😊🐢🍕"
        "  </text>"
        "</svg>";

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
    Magick::InitializeMagick(nullptr);

    cppcoro::static_thread_pool thread_pool(4);
    const redis_queue queue {redis_host, 6379, thread_pool};

    while (true) {
        auto r = cppcoro::sync_wait(queue.dequeue(queue_name, 0));
        generate(test_svg, queue);
    }

    spdlog::drop_all();
    return EXIT_SUCCESS;
}
