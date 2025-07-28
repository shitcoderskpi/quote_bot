
#include <csignal>
#include <string>
#include <spdlog/spdlog.h>

#include "global.h"
#include "redis_queue.h"
#include "prometheus/exposer.h"
#include "prometheus/registry.h"
#include <Magick++.h>

#include "cppcoro/sync_wait.hpp"

void sig_handler(const int sig) {
    switch (sig) {
        case SIGINT:
        case SIGTERM:
            spdlog::info("Received SIGINT/SIGTERM, exiting...");
            spdlog::drop_all();
            exit(EXIT_SUCCESS);
        default:
            spdlog::info("Received signal {}", sig);
    }
}

void generate(const std::string& bg, const std::string &pango, const redis_queue& queue) {
    try {
        Magick::Image bg_img;
        Magick::Image pango_img;

        bg_img.read(Magick::Blob(bg.data(), bg.size()));
        bg_img.density(Magick::Point(300, 300));

        pango_img.read(pango);
        pango_img.defineSet("pango:markup", "true");
        pango_img.density(Magick::Point(300, 300));
        pango_img.textEncoding("UTF-8");

        bg_img.composite(pango_img, MagickCore::GravityType::CenterGravity, Magick::OverCompositeOp);
        bg_img.magick("PNG");
        Magick::Blob blob;
        bg_img.write(&blob);

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
        "</svg>";

    const std::string test_text = "pango:<span font_family='JetBrains Mono Nerd Font, Noto Color Emoji' size='48000'>"
        "Emoji: 😎🦾✨"
        "</span>";

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
        generate(test_svg, test_text, queue);
    }

    spdlog::drop_all();
    return EXIT_SUCCESS;
}
