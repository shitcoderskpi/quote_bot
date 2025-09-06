//
// Created by mmatz on 8/4/25.
//

#ifndef TEXT_H
#define TEXT_H
#include <sstream>
#include <string>
#include <pango/pango.h>

namespace pango {

    // See: https://docs.gtk.org/Pango/enum.Weight.html
    enum weight : unsigned short {
        thin = 100,
        ultralight = 200,
        light = 300,
        semilight = 350,
        book = 380,
        normal = 400,
        medium = 500,
        semibold = 600,
        bold = 700,
        ultrabold = 800,
        heavy = 900,
        ultraheavy = 1000,
    };

    // See: https://docs.gtk.org/Pango/pango_markup.html
    struct text {
        std::string content;
        int x;
        int y;
        // Font attributes
        std::string font_family;
        std::string size;
        unsigned short weight;
        PangoAlignment alignment;
        std::string color;
        int wrap_width = 0;
        PangoWrapMode wrap_mode = PANGO_WRAP_WORD;

        [[nodiscard]] std::string font_description() const {
            std::stringstream ss;
            ss << font_family << " " << size;
            return ss.str();
        }
    };

    inline std::string to_string(const text& t) {
        std::stringstream ss;

        ss << "<span font_desc=\"" << t.font_family << " " << t.size << "\" font_weight=\"" << t.weight << "\" color=\"" << t.color << "\">" << t.content << "</span>";
        return ss.str();
    }

    inline std::ostream& operator<<(std::ostream& os, const text& t) {
        os << "<span font_desc=\"" << t.font_family << " " << t.size << "\" font_weight=\"" << t.weight << "\" color=\"" << t.color << "\">" << t.content << "</span>";
        return os;
    }

} // pango

#endif //TEXT_H
