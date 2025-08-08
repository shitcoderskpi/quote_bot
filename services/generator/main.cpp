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

volatile sig_atomic_t shutdown = 0;

void sig_handler(const int sig) {
    spdlog::info("Received signal {}, exiting...", strsignal(sig));
    spdlog::drop_all();
    shutdown = 1;
}

long long get_count_of_files_in_directory(const std::filesystem::path &dir) {
    if (!std::filesystem::exists(dir) || !std::filesystem::is_directory(dir)) {
        return -1;
    }

    try {
        std::filesystem::directory_iterator it;
        return std::count_if(
            std::filesystem::begin(it),
            std::filesystem::end(it),
            [](const std::filesystem::directory_entry &entry) {
                return std::filesystem::is_regular_file(entry);
                }
            );
    } catch (const std::filesystem::filesystem_error &e) {
        spdlog::error("Filesystem error: {}", e.what());
        return -1;
    }
}

constexpr std::string_view colorspace_type_to_string(const Magick::ColorspaceType &cs) {
    switch (cs) {
        case Magick::UndefinedColorspace: return "Undefined";
        case Magick::RGBColorspace: return "RGB";
        case Magick::GRAYColorspace: return "Gray";
        case Magick::TransparentColorspace: return "Transparent";
        case Magick::OHTAColorspace: return "OHTA";
        case Magick::LabColorspace: return "Lab";
        case Magick::XYZColorspace: return "XYZ";
        case Magick::YCbCrColorspace: return "YCbCr";
        case Magick::YCCColorspace: return "YCC";
        case Magick::YIQColorspace: return "YIQ";
        case Magick::YPbPrColorspace: return "YPbPr";
        case Magick::YUVColorspace: return "YUV";
        case Magick::CMYKColorspace: return "CMYK";
        case Magick::sRGBColorspace: return "sRGB";
        case Magick::HSLColorspace: return "HSL";
        case Magick::HWBColorspace: return "HWB";
        case Magick::Rec601YCbCrColorspace: return "Rec601 YCbCr";
        case Magick::Rec709YCbCrColorspace: return "Rec709 YCbCr";
        case Magick::LogColorspace: return "Log";
        case Magick::ColorspaceType::DisplayP3Colorspace: return "DCI/P3";
        case Magick::ColorspaceType::Adobe98Colorspace: return "AdobeRGB 98";
        default: return "Unknown";
    }
}

int main()
{
    using namespace prometheus;
    constexpr auto prometheus_host = "0.0.0.0:8080";
    const std::string queue_name = "generate:jobs";
    constexpr std::string_view templates_path = "../templates";
    constexpr std::string_view cache_path = "../.cache/templates";

    if (!std::filesystem::exists(cache_path)) {
        std::filesystem::create_directories(cache_path);
    }

    std::signal(SIGINT, sig_handler);
    std::signal(SIGTERM, sig_handler);

    spdlog::set_level(spdlog::level::debug);
    const auto logger = logger_init("main");
    logger->info("Logging setup is done.");

    // FIXME:
    ////////////////////////////////////////////////////////////////////
    ///////// THERE IS SOME WIERD ISSUE WITH `civetweb` WORKER /////////
    /////////                 IT SEGFAULTS                     /////////
    /////////            ONLY IN DOCKER CONTAINER              /////////
    /////////     BACKTRACE IN `infra/core/backtrace.txt`      /////////
    ////////////////////////////////////////////////////////////////////
    // Exposer exposer {prometheus_host};
    // logger->debug("Exposed prometheus on {}", prometheus_host);
    //
    // // metrics registry
    // auto registry = std::make_shared<Registry>();
    //
    // exposer.RegisterCollectable(registry);

    const auto redis_host = getenv("REDIS_HOST");
    if (redis_host == nullptr) {
        logger->critical("REDIS_HOST is not set in the environment.");
        exit(EXIT_FAILURE);
    }

    // TODO: use cli args to set number of threads
    cppcoro::static_thread_pool thread_pool {4};
    templates::storage storage {};
    if (get_count_of_files_in_directory(cache_path) > 0) {
        cppcoro::sync_wait( storage.load_templates_async(cache_path, thread_pool) );
    } else {
        cppcoro::sync_wait( storage.load_templates_async(templates_path, thread_pool) );
    }

    renderer r {};
    const redis_queue queue {redis_host, 6379, thread_pool};
    logger->info("Redis started at {}:{}", redis_host, 6379);

    while (!shutdown) {
        auto reply = cppcoro::sync_wait(queue.dequeue(queue_name, 0));
        auto result = r.render_image(storage["tg_template"], Magick::Point {300, 300});

        result.quantizeColorSpace(Magick::ColorspaceType::DisplayP3Colorspace);
        result.depth(8);
        result.quantize(true);
        result.magick("PNG32");

        logger->debug("Image dimensions: x={}|y={}", result.columns(), result.rows());
        logger->debug("Using {} bits per channel, color space: {}, alpha channel: {}", result.depth(), colorspace_type_to_string(result.colorSpace()), result.alpha());

        logger->debug("Enqueuing data to results queue");
        result.write("output.png");
        cppcoro::sync_wait(queue.enqueue("generate:results", image_to_base64(result)));
    }

    storage.save_all(cache_path);
    spdlog::drop_all();
    return EXIT_SUCCESS;
}
