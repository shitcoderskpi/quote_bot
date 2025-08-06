//
// Created by mmatz on 8/4/25.
//

#include "text_factory.h"

namespace pango {
    text text_factory::from_node(const pugi::xml_node &node) {
        text text;
        text.x = node.attribute("x").as_int();
        text.y = node.attribute("y").as_int();
        text.size = node.attribute("font-size").as_string();
        text.weight = font_weight_to_pango(node.attribute("font-weight"));
        text.alignment = anchor_to_alignment(node.attribute("text-anchor").as_string());
        text.color = node.attribute("fill").as_string();
        text.content = node.child_value();

        return text;
    }

    text text_factory::from_capnp_reader(const Image::TextEntry::Reader &reader) {
        text text;
        text.content = reader.getContent();
        text.x = reader.getX();
        text.y = reader.getY();
        text.size = reader.getSize();
        text.weight = reader.getWeight();
        text.alignment = capnp_to_pango_alignment(reader.getAlignment());
        text.color = reader.getColor();
        text.wrap_width = reader.getWrapWidth();

        return text;
    }

    PangoAlignment text_factory::anchor_to_alignment(const std::string &anchor) {
        if (!alignment_map.contains(anchor)) {
            return PANGO_ALIGN_LEFT;
        }

        return alignment_map.at(anchor);
    }

    unsigned short text_factory::font_weight_to_pango(const pugi::xml_attribute &weight) {
        if (weight.as_int() == 0) {
            if (!weight_map.contains(weight.as_string())) {
                return normal;
            }

            return weight_map.at(weight.as_string());
        }

        return weight.as_int();
    }

    PangoAlignment text_factory::capnp_to_pango_alignment(const Image::TextEntry::Alignment &a) {
        switch (a) {
            case Image::TextEntry::Alignment::LEFT: return PANGO_ALIGN_LEFT;
            case Image::TextEntry::Alignment::CENTER: return PANGO_ALIGN_CENTER;
            case Image::TextEntry::Alignment::RIGHT: return PANGO_ALIGN_RIGHT;
            default: return PANGO_ALIGN_LEFT;
        }
    }
} // pango