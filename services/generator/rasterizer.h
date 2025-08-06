//
// Created by mmatz on 8/6/25.
//

#ifndef RASTERIZER_H
#define RASTERIZER_H
#include <array>
#include <cairo.h>
#include <memory>
#include <string>
#include <Magick++/Geometry.h>
#include <spdlog/spdlog.h>

#include "text.h"

namespace pango {

    struct raster_text {
        int width;
        int height;
        cairo_surface_t *surface;
        unsigned char* data;
        unsigned long stride;

        raster_text(int width, int height, cairo_surface_t *surface, unsigned char *data, int stride) noexcept;
        ~raster_text() noexcept;
    };

    // TODO: add support for font families
    class rasterizer {
        public:
        rasterizer() noexcept;
        ~rasterizer() noexcept;

        static raster_text raster(const text &, const Magick::Point &scale);

    private:
        std::shared_ptr<spdlog::logger> logger;
        static constexpr std::array<unsigned char, 4> red {255, 255, 0, 0};
        static constexpr std::array<std::array<unsigned char, 4>, 6> debug_colors {
            red,
            {255, 0, 0, 255},
            {255, 0, 255, 0},
            {255, 178, 255, 255},
            {255, 255, 255, 0},
            {255, 239, 191, 4}
        };

        // TODO: use dynamic bases (e.g. using font size config ???)
        static constexpr int base_width = 300;
        static constexpr int base_height = 50;

    };

} // pango

#endif //RASTERIZER_H
