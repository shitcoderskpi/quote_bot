//
// Created by mmatz on 8/4/25.
//

#ifndef TEXT_FACTORY_H
#define TEXT_FACTORY_H
#include <pugixml.hpp>
#include <unordered_map>

#include "text.h"

namespace pango {

    class text_factory {
    public:
        static text from_node(const pugi::xml_node& node);

    private:
        static inline const std::unordered_map<std::string, alignment> alignment_map{
                            {"middle", center},
                            {"end", right}
        };
        static inline const std::unordered_map<std::string, unsigned short> weight_map{
                    {"lighter", ultralight},
                    {"light", light},
                    {"bold", bold},
                    {"bolder", ultrabold}
        };

        static alignment anchor_to_alignment(const std::string &anchor);

        static unsigned short font_weight_to_pango(const pugi::xml_attribute &weight);
    };

} // pango

#endif //TEXT_FACTORY_H
