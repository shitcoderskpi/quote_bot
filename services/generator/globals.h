//
// Created by mmatz on 7/15/25.
//

#ifndef GLOBALS_H
#define GLOBALS_H
#include <memory>
#include <Magick++/Blob.h>
#include <Magick++/Image.h>
#include <spdlog/spdlog.h>
#include <spdlog/sinks/stdout_color_sinks.h>
#include <openssl/bio.h>
#include <openssl/evp.h>

inline std::shared_ptr<spdlog::logger> logger_init(const std::string &name,
                                                   const spdlog::level::level_enum lvl = spdlog::level::debug,
                                                   const std::string &pattern = "[%Y-%m-%d %H:%M:%S.%e %^%l%$] %n: %v") {
    const auto logger = spdlog::stdout_color_mt(name);
    logger->set_level(lvl);
    logger->set_pattern(pattern);
    return logger;
}

// TODO: may be broken. Needs further testing
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

// TODO: may be broken. Needs further testing
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
