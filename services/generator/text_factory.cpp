//
// Created by mmatz on 8/4/25.
//

#include "text_factory.h"

#include "globals.h"

namespace pango {
    text text_factory::from_node(const pugi::xml_node &node, const std::string &fallback_font) {
        text text;
        text.x = node.attribute("x").as_int();
        text.y = node.attribute("y").as_int();
        text.font_family = get_font_family(node.attribute("font-family"), fallback_font);
        text.size = node.attribute("font-size").as_string();
        text.weight = font_weight_to_pango(node.attribute("font-weight"));
        text.alignment = anchor_to_alignment(node.attribute("text-anchor").as_string());
        text.color = trim(node.attribute("fill").as_string());
        text.content = trim(node.child_value());

        return text;
    }

    PangoAlignment text_factory::anchor_to_alignment(const std::string &anchor) {
        if (!alignment_map.contains(trim(anchor))) {
            return PANGO_ALIGN_LEFT;
        }

        return alignment_map.at(trim(anchor));
    }

    unsigned short text_factory::font_weight_to_pango(const pugi::xml_attribute &weight) {
        if (weight.as_int() == 0) {
            if (!weight_map.contains(trim(weight.as_string()))) {
                return normal;
            }

            return weight_map.at(trim(weight.as_string()));
        }

        return weight.as_int();
    }

    std::string text_factory::get_font_family(const pugi::xml_attribute &attr, const std::string &fallback) {
        if (strcmp(attr.as_string(), "") == 0) return fallback;
        return trim(attr.as_string());
    }
} // pango