//
// Created by mmatz on 8/6/25.
//

#ifndef RASTERIZER_H
#define RASTERIZER_H
#include <Magick++/Geometry.h>
#include <array>
#include <cairo.h>
#include <memory>
#include <spdlog/spdlog.h>

#include "text.h"

namespace pango {

    struct font_metrics {
        explicit font_metrics(PangoFontMetrics *metrics) noexcept : metrics(metrics) {}
        ~font_metrics() noexcept;

        [[nodiscard]] int ascent() const noexcept;
        [[nodiscard]] int descent() const noexcept;
        [[nodiscard]] int height() const noexcept;
        [[nodiscard]] int ascent_px() const noexcept;
        [[nodiscard]] int descent_px() const noexcept;
        [[nodiscard]] int height_px() const noexcept;

    private:
        PangoFontMetrics *metrics;
    };

    struct raster_text {
        int width;
        int height;
        cairo_surface_t *surface;
        unsigned char *data;
        unsigned long stride;
        font_metrics metrics;

        raster_text(int width, int height, cairo_surface_t *surface, PangoFontMetrics *metrics, unsigned char *data,
                    int stride) noexcept;

        ~raster_text() noexcept;
    };

    class rasterizer {
    public:
        rasterizer() noexcept;
        ~rasterizer() noexcept;

        static raster_text raster(const text &, const Magick::Point &, bool debug_paint = false);

        static Magick::Point calculate_size(const text &, const Magick::Point &);

    private:
        std::shared_ptr<spdlog::logger> logger;
#ifdef DEBUG
        static constinit inline int _color_index{};
        static constexpr std::array<unsigned char, 4> red{255, 255, 0, 0};
        static constexpr std::array<std::array<unsigned char, 4>, 6> debug_colors{
                red, {255, 0, 0, 255}, {255, 0, 255, 0}, {255, 178, 255, 255}, {255, 255, 255, 0}, {255, 239, 191, 4}};
#endif
    };

} // namespace pango

#endif // RASTERIZER_H
