//
// Created by mmatz on 8/4/25.
//

#ifndef PREPROCESSOR_H
#define PREPROCESSOR_H
#include <pugixml.hpp>
#include <spdlog/spdlog.h>
#include <stdexcept>
#include <string>

#include "exceptions.h"
#include "globals.h"
#include "image.h"
#include "text_factory.h"

namespace templates {

    class preprocessor {
    public:
        preprocessor() noexcept;
        ~preprocessor() noexcept = default;

        [[nodiscard]] static image preprocess(const std::string &input, const std::string &fallback_font);

    private:
        std::shared_ptr<spdlog::logger> logger;
    };

    inline preprocessor::preprocessor() noexcept : logger(logger_init("preprocessor")) {}

    inline image preprocessor::preprocess(const std::string &input, const std::string &fallback_font) {
        pugi::xml_document doc;
        if (const pugi::xml_parse_result result = doc.load_string(input.data()); !result) {
            throw exceptions::templates::preprocessor_error(std::format("Failed to parse XML document: {}", result.description()));
        }

        std::string default_font = fallback_font;

        if (pugi::xml_node svg_node = doc.select_node("//*[local-name()='svg']").node();
            !trim(svg_node.attribute("font-family").as_string()).empty()) {
            default_font = trim(svg_node.attribute("font-family").as_string());
        }

        const auto nodes = doc.select_nodes("//*[local-name()='text']");
        std::vector<pango::text> text_entries {nodes.size()};

        size_t i {};
        for (const auto& xpath_nodes = nodes;
             const pugi::xpath_node &entry: xpath_nodes) {
            const pugi::xml_node &node{entry.node()};

            auto const text = pango::text_factory::from_node(node, default_font);

            node.parent().remove_child(node);
            text_entries[i] = text;
            ++i;
        }

        std::stringstream ss;
        doc.save(ss, "  ", pugi::format_no_declaration | pugi::format_indent);

        return image{ss.str(), text_entries};
    }

} // namespace templates

#endif // PREPROCESSOR_H
