//
// Created by mmatz on 8/10/25.
//

#ifndef COMPRESSOR_H
#define COMPRESSOR_H
#include <string>
#include <zstd.h>


class compressor {
public:
    compressor() noexcept = default;
    ~compressor() noexcept = default;

    static std::string compress(const std::string &, int);
    static std::string decompress(const std::string &);

    static size_t estimate_compressed_size(const std::string &);

};

inline std::string compressor::compress(const std::string &input, const int compression_level = 5) {
    size_t const max_compressed_size = ZSTD_compressBound(input.size());
    std::string compressed;
    compressed.resize(max_compressed_size);

    size_t const compressed_size = ZSTD_compress(
        compressed.data(), compressed.size(),
        input.data(), input.size(),
        compression_level);

    if (ZSTD_isError(compressed_size)) {
        return "";
    }

    compressed.resize(compressed_size);
    return compressed;
}

inline std::string compressor::decompress(const std::string &input) {
    std::string decompressed;

    size_t const compressed_size = ZSTD_getFrameContentSize(input.data(), input.size());
    if (ZSTD_isError(compressed_size)) {
        return "";
    }

    decompressed.resize(compressed_size);

    size_t const decompressed_size = ZSTD_decompress(
        decompressed.data(), decompressed.size(),
        input.data(), input.size()
        );
    if (ZSTD_isError(decompressed_size)) {
        return "";
    }

    decompressed.resize(decompressed_size);
    return decompressed;
}

inline size_t compressor::estimate_compressed_size(const std::string &input) {
    return ZSTD_compressBound(input.size());
}


#endif //COMPRESSOR_H
