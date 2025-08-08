//
// Created by mmatz on 8/4/25.
//

#ifndef RENDERER_H
#define RENDERER_H
#include <Magick++/Functions.h>
#include <Magick++/Image.h>
#include <Magick++/Include.h>
#include <memory>
#include <spdlog/logger.h>

#include "globals.h"
#include "image.h"
#include "rasterizer.h"


template<typename T>
T clamp(const T &min, const T &value, const T &max) {
    return std::max(min, std::min(value, max));
}


class renderer {
public:
    renderer() noexcept;
    ~renderer() noexcept = default;

    [[nodiscard]] Magick::Image render_image(const templates::image &img, const Magick::Point &density) const;

private:
    std::shared_ptr<spdlog::logger> logger;

    static Magick::Point calculate_offsets(const Magick::Image &t_img, const Magick::Image &bg, const pango::text &text, const pango::raster_text &r_text, const
                                           Magick::Point &scale);

    static double calculate_baseline(const pango::raster_text &raster, const Magick::Point &scale);

    static long alignment_to_offset(const PangoAlignment &alignment, const long &text_width);
};

inline renderer::renderer() noexcept {
    Magick::InitializeMagick(nullptr);
    logger = logger_init("renderer");
}

inline Magick::Image renderer::render_image(const templates::image &img, const Magick::Point &density = Magick::Point(300, 300)) const {
    Magick::Image bg;
    bg.backgroundColor(Magick::Color("transparent"));
    bg.density(density);
    bg.read(Magick::Blob {img.background.data(), img.background.size()});

    const Magick::Point scale {density.x() / 96, density.y() / 96};

    logger->debug("Img scale: x={}|y={}", scale.x(), scale.y());

    for (const auto &entry : img.text_entries) {
        const auto result = pango::rasterizer::raster(entry, scale);

        Magick::Image text;
        text.density(density);
        text.read(result.width, result.height, "BGRA", Magick::CharPixel, result.data);
        text.trim();

        const auto offset = calculate_offsets(text, bg, entry, result, scale);

        bg.composite(text, Magick::Geometry{0, 0, static_cast<long int>(offset.x()), static_cast<long int>(offset.y())}, Magick::OverCompositeOp);
    }

    return bg;
}

inline Magick::Point renderer::calculate_offsets(const Magick::Image &t_img, const Magick::Image &bg, const pango::text &text, const pango::raster_text& r_text,
    const Magick::Point &scale) {

    const double x_offset = clamp(
        0.0,
        text.x * scale.x() + alignment_to_offset(text.alignment, static_cast<long>(t_img.columns())),
        static_cast<double>(bg.columns())
        );
    const double y_offset = clamp(
        0.0,
        text.y * scale.y() + calculate_baseline(r_text, scale),
        static_cast<double>(bg.rows())
        );

    return {x_offset, y_offset};
}

inline double renderer::calculate_baseline(const pango::raster_text &raster, const Magick::Point &scale) {
    const double ascent  = raster.metrics.ascent()  * scale.y();
    const double descent = raster.metrics.descent() * scale.y();

    return -std::round(ascent - (ascent + descent) / 2.0 + descent * 0.5);
}

inline long renderer::alignment_to_offset(const PangoAlignment &alignment, const long &text_width) {
    switch (alignment) {
        case PANGO_ALIGN_CENTER:
            return - text_width / 2;
        case PANGO_ALIGN_RIGHT:
            return -text_width;
        default:
            return 0;
    }
}


#endif //RENDERER_H
