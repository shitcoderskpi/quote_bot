//
// Created by mmatz on 8/6/25.
//

#include "rasterizer.h"

#include <Magick++/Geometry.h>
#include <Magick++/Include.h>
#include <cairo.h>
#include <pango/pangocairo.h>
#include <spdlog/spdlog.h>

#include "globals.h"
#include "text.h"

namespace pango {
    font_metrics::~font_metrics() noexcept {
        if (metrics)
            pango_font_metrics_unref(metrics);
    }

    raster_text::raster_text(const int width, const int height, cairo_surface_t *surface, PangoFontMetrics *metrics,
                             unsigned char *data) noexcept :
        width(width), height(height), metrics(metrics) {
        if (surface == nullptr || data == nullptr) {
            throw std::invalid_argument("raster_text::raster_text: null surface");
        }
        this->surface = surface;
        this->data = {data, static_cast<size_t>(width * height)};
    }

    raster_text::~raster_text() noexcept {
        if (surface)
            cairo_surface_destroy(surface);
    }

    Magick::Point operator/(const Magick::Point &lhs, const double &rhs) {
        return Magick::Point{lhs.x() / rhs, lhs.y() / rhs};
    }

#ifdef DEBUG
    raster_text rasterizer::raster(const text &t, const Magick::Point &scale, const bool debug_paint) {
#else
    raster_text rasterizer::raster(const text &t, const Magick::Point &scale) {
#endif
        cairo_surface_t *dummy = cairo_image_surface_create(CAIRO_FORMAT_ARGB32, 1, 1);
        cairo_t *dummy_cr = cairo_create(dummy);

        PangoLayout *layout = pango_cairo_create_layout(dummy_cr);
        const auto markup = to_string(t);
        pango_layout_set_markup(layout, markup.c_str(), markup.length());

        PangoFontDescription *font = pango_font_description_from_string(t.font_description().c_str());
        pango_layout_set_font_description(layout, font);

        PangoContext *context = pango_layout_get_context(layout);
        PangoFontMetrics *metrics = pango_context_get_metrics(context, font, pango_context_get_language(context));

        int width, height;
        pango_layout_get_pixel_size(layout, &width, &height);
        const int surf_width = static_cast<double>(width) * scale.x() + WIDTH_PADDING;
        const int surf_height = static_cast<double>(height) * scale.y();

        pango_font_description_free(font);
        cairo_surface_destroy(dummy);
        cairo_destroy(dummy_cr);

        const auto surface = cairo_image_surface_create(CAIRO_FORMAT_ARGB32, surf_width, surf_height);
        const auto cr = cairo_create(surface);

#ifdef DEBUG
        if (debug_paint) {
            const auto color = debug_colors.at(_color_index++ % debug_colors.size());
            cairo_set_source_rgba(cr, static_cast<double>(color.at(1)) / 255, static_cast<double>(color.at(2)) / 255,
                                  static_cast<double>(color.at(3)) / 255, static_cast<double>(color.at(0)) / 255);
            cairo_paint(cr);
        }
#endif

        cairo_scale(cr, scale.x(), scale.y());
        pango_layout_set_width(layout, surf_width * PANGO_SCALE);
        pango_cairo_update_context(cr, pango_layout_get_context(layout));
        pango_cairo_update_layout(cr, layout);
        pango_cairo_show_layout(cr, layout);

        g_object_unref(layout);

        cairo_surface_flush(surface);
        const auto data = cairo_image_surface_get_data(surface);

        return raster_text{surf_width, surf_height, surface, metrics, data};
    }
} // namespace pango
