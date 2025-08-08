//
// Created by mmatz on 8/4/25.
//

#ifndef PREPROCESSOR_H
#define PREPROCESSOR_H
#include <string>
#include <pugixml.hpp>
#include <spdlog/spdlog.h>

#include "globals.h"
#include "image.h"
#include "text_factory.h"

namespace templates {

    class preprocessor {
        public:
        preprocessor() noexcept;
        ~preprocessor() noexcept = default;

        [[nodiscard]] image preprocess(const std::string &input, const std::string &fallback_font) const;

        private:
        std::shared_ptr<spdlog::logger> logger;

    };

    inline preprocessor::preprocessor() noexcept : logger(logger_init("preprocessor")) {}

    inline image preprocessor::preprocess(const std::string &input, const std::string &fallback_font) const {
        pugi::xml_document doc;
        if (const pugi::xml_parse_result result = doc.load_string(input.c_str()); !result) {
            logger->error("Failed to parse XML document: {}", result.description());
            return {input, {}};
        }

        std::string default_font = fallback_font;

        if (pugi::xml_node svg_node = doc.select_node("//svg:svg").node(); trim(svg_node.attribute("font-family").as_string()) != "") {
            default_font = trim(svg_node.attribute("font-size").as_string());
        }

        std::vector<pango::text> text_entries;

        for (const auto xpath_nodes = doc.select_nodes("//*[local-name()='text']"); const pugi::xpath_node& entry: xpath_nodes) {
            const pugi::xml_node& node {entry.node()};

            auto const text = pango::text_factory::from_node(node, default_font);

            node.parent().remove_child(node);
            text_entries.emplace_back(text);
        }

        std::stringstream ss;
        doc.save(ss, "  ", pugi::format_no_declaration | pugi::format_indent);

        return image { ss.str(), text_entries };
    }
} // templates

#endif //PREPROCESSOR_H
