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
#include <unistd.h>

#include "Magick++/Color.h"
#include "globals.h"
#include "image.h"


class renderer {
public:
    renderer() noexcept;
    ~renderer() noexcept = default;

    Magick::Image render_image(const templates::image &img, const Magick::Point &density) const;

private:
    std::shared_ptr<spdlog::logger> logger;

};

inline renderer::renderer() noexcept {
    Magick::InitializeMagick(nullptr);
    logger = logger_init("renderer");
}

inline Magick::Image renderer::render_image(const templates::image &img, const Magick::Point &density = Magick::Point(300, 300)) const {
    Magick::Image bg;
    bg.density(density);
    bg.read(Magick::Blob {img.background.data(), img.background.size()});

    const double w_coeff = density.x() / 96;
    const double h_coeff = density.y() / 96;

    logger->debug("Img coefficients: width={}|height={}", w_coeff, h_coeff);

    for (const auto &entry : img.text_entries) {
        Magick::Image text;

        text.font("Noto Font");
        text.density(density);
        text.read(pango::to_string(entry));
        text.defineSet("pango:markup", "true");
        text.transparent(Magick::Color("white"));
        text.textEncoding("UTF-8");

        const double text_height = text.rows();
        const double text_width = text.columns();
        
        const double baseline_offset = text_height * 0.85;
        
        const int x_offset = static_cast<ssize_t>(entry.x * w_coeff);
        const int y_offset = static_cast<ssize_t>(entry.y * h_coeff - baseline_offset);

        logger->debug("Text positioning: x={}|y={} | Rendered size: {}x{}", 
                     x_offset, y_offset, text_width, text_height);

        bg.composite(text, Magick::Geometry{0, 0, x_offset, y_offset}, Magick::OverCompositeOp);
    }

    return bg;
}


#endif //RENDERER_H
