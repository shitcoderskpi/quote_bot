#ifndef GENERATOR_PARSER_H
#define GENERATOR_PARSER_H

#include <expected>
#include <string>
#include <string_view>
#include <vector>
#include <pango/pango.h>

struct svg_message {
    std::string svg_data;
};

struct pango_message {
    int x;
    int y;
    int wrap_width;
    PangoWrapMode wrap_mode;
    PangoAlignment alignment;
    std::string font_description;
    std::string markup;
};

struct metadata_message {
    std::string chat_id;
};

struct parsed_message {
    metadata_message metadata;
    svg_message svg;
    std::vector<pango_message> pangos;
};

class parser {
public:
    static std::expected<parsed_message, std::string> parse(std::string_view payload);
};

#endif // GENERATOR_PARSER_H
