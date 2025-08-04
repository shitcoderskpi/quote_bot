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
        text.alignment_ = anchor_to_alignment(node.attribute("text-anchor").as_string());
        text.color = node.attribute("fill").as_string();
        text.content = node.child_value();

        return text;
    }

    alignment text_factory::anchor_to_alignment(const std::string &anchor) {
        if (!alignment_map.contains(anchor)) {
            return left;
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
} // pango