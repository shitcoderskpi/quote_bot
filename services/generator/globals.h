//
// Created by mmatz on 7/15/25.
//

#ifndef GLOBALS_H
#define GLOBALS_H
#include <csignal>
#include <memory>
#include <Magick++/Blob.h>
#include <Magick++/Image.h>
#include <spdlog/spdlog.h>
#include <spdlog/sinks/stdout_color_sinks.h>
#include <openssl/bio.h>
#include <openssl/evp.h>
#include <immintrin.h>
#include <execution>
#include <filesystem>

inline volatile sig_atomic_t shutdown = 0;

inline void sig_handler(const int sig) {
    spdlog::info("Received signal {}, exiting...", strsignal(sig));
    spdlog::drop_all();
    shutdown = 1;
}

inline long long get_count_of_files_in_directory(const std::filesystem::path &dir) {
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

constexpr std::string trim(const std::string_view &sv, const std::string_view &what = " \t\n\r") noexcept {
    const auto begin = sv.find_first_not_of(what);
    if (begin == std::string_view::npos) return {};
    const auto end = sv.find_last_not_of(what);
    return std::string(sv.substr(begin, end - begin + 1));
}

constexpr std::string trim(const char* s, const std::string_view& what = " \t\n\r") {
    return trim(std::string_view(s ? s : ""), what);
}

constexpr std::string trim(const std::string& s, const std::string_view &what = " \t\n\r") {
    return trim(std::string_view(s), what);
}

inline std::shared_ptr<spdlog::logger> logger_init(const std::string &name,
                                                   const spdlog::level::level_enum lvl = spdlog::level::debug,
                                                   const std::string &pattern = "[%Y-%m-%d %H:%M:%S.%e %^%l%$] %n: %v"){
    if (spdlog::get(name) != nullptr) return spdlog::get(name);

    const auto logger = spdlog::stdout_color_mt(name);
    logger->set_level(lvl);
    logger->set_pattern(pattern);
    return logger;
}

inline std::string image_to_base64(const Magick::Blob& blob) {
    BIO *b64 = BIO_new(BIO_f_base64());
    BIO *bio = BIO_new(BIO_s_mem());
    bio = BIO_push(b64, bio);

    BIO_set_flags(bio, BIO_FLAGS_BASE64_NO_NL);
    BIO_write(bio, blob.data(), static_cast<int>(blob.length()));
    BIO_flush(bio);

    char* bufferPtr;
    const long len = BIO_get_mem_data(bio, &bufferPtr);
    std::string result(bufferPtr, len);

    BIO_free_all(bio);
    return result;
}

inline std::string image_to_base64(Magick::Image& img) {
    Magick::Blob blob;
    img.write(&blob);

    BIO *b64 = BIO_new(BIO_f_base64());
    BIO *bio = BIO_new(BIO_s_mem());
    bio = BIO_push(b64, bio);

    BIO_set_flags(bio, BIO_FLAGS_BASE64_NO_NL);
    BIO_write(bio, blob.data(), static_cast<int>(blob.length()));
    BIO_flush(bio);

    char* bufferPtr;
    const long len = BIO_get_mem_data(bio, &bufferPtr);
    std::string result(bufferPtr, len);

    BIO_free_all(bio);
    return result;
}

#endif //GLOBALS_H
