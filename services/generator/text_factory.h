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
        static text from_node(const pugi::xml_node &node, const std::string &fallback_font);


    private:
        static inline const std::unordered_map<std::string, PangoAlignment> alignment_map{
                            {"middle", PANGO_ALIGN_CENTER},
                            {"end", PANGO_ALIGN_RIGHT}
        };
        static inline const std::unordered_map<std::string, unsigned short> weight_map{
                    {"lighter", ultralight},
                    {"light", light},
                    {"bold", bold},
                    {"bolder", ultrabold}
        };

        static PangoAlignment anchor_to_alignment(const std::string &anchor);

        static unsigned short font_weight_to_pango(const pugi::xml_attribute &weight);

        // static PangoAlignment capnp_to_pango_alignment(const Image::TextEntry::Alignment &a);

        static std::string get_font_family(const pugi::xml_attribute &attr, const std::string &fallback);
    };

} // pango

#endif //TEXT_FACTORY_H
