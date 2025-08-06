//
// Created by mmatz on 8/6/25.
//

#include "rasterizer.h"

#include <cairo.h>
#include <Magick++/Geometry.h>
#include <Magick++/Include.h>
#include <pango/pangocairo.h>
#include <spdlog/spdlog.h>

#include "globals.h"
#include "text.h"

namespace pango {

    raster_text::raster_text(const int width, const int height, cairo_surface_t *surface, unsigned char *data,
        const int stride) noexcept {
        if (surface == nullptr || data == nullptr) {
            return;
        }
        this->width = width;
        this->height = height;
        this->surface = surface;
        this->data = data;
        this->stride = stride;
    }

    raster_text::~raster_text() noexcept {
        if (surface != nullptr) cairo_surface_destroy(surface);
    }

    rasterizer::rasterizer() noexcept : logger(logger_init("rasterizer")) {}

    rasterizer::~rasterizer() noexcept = default;

    Magick::Point operator/(const Magick::Point & lhs, const double & rhs) {
        return Magick::Point {lhs.x() / rhs, lhs.y() / rhs};
    }

    raster_text rasterizer::raster(const text &t, const Magick::Point &scale) {
        const int surf_width = static_cast<int>(base_width * scale.x());
        const int surf_height = static_cast<int>(base_height * scale.y());

        cairo_surface_t* surface = cairo_image_surface_create(CAIRO_FORMAT_ARGB32, surf_width, surf_height);
        cairo_t* cr = cairo_create(surface);

        cairo_scale(cr, scale.x(), scale.y());

        cairo_set_source_rgba(cr, 0.0, 0.0, 0.0, 0.0);
        cairo_paint(cr);

        PangoLayout* layout = pango_cairo_create_layout(cr);
        const auto markup = to_string(t);

        PangoFontDescription* font = pango_font_description_from_string("sans-serif");
        pango_layout_set_font_description(layout, font);
        pango_font_description_free(font);

        pango_layout_set_markup(layout, markup.c_str(), markup.length());
        pango_layout_set_width(layout, surf_width * PANGO_SCALE);
        pango_cairo_show_layout(cr, layout);

        g_object_unref(layout);
        cairo_destroy(cr);

        cairo_surface_flush(surface);
        const auto data = cairo_image_surface_get_data(surface);
        const int stride = cairo_image_surface_get_stride(surface);

        return raster_text {surf_width, surf_height, surface, data, stride};
    }
} // pango