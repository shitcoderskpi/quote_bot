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

    struct string_writer final : pugi::xml_writer {
        std::string &out;
        explicit string_writer(std::string &o) : out(o) {}
        void write(const void* data, const size_t size) override {
            out.append(static_cast<const char*>(data), size);
        }
    };

    static bool node_has_local_name_text(const pugi::xml_node& n) {
        const char* nm = n.name();
        if (!nm || nm[0] == '\0') return false;
        const char* colon = std::strrchr(nm, ':');
        if (!colon) return std::strcmp(nm, "text") == 0;
        return std::strcmp(colon + 1, "text") == 0;
    }


    class preprocessor {
        public:
        preprocessor() noexcept;
        ~preprocessor() noexcept = default;

        [[nodiscard]] image preprocess(const std::string &input, const std::string &fallback_font) const;
        // [[nodiscard]] image o_preprocess(const std::string &input, const std::string &fallback_font, bool allow_parallel) const;

        private:
        std::shared_ptr<spdlog::logger> logger;

    };

    inline preprocessor::preprocessor() noexcept : logger(logger_init("preprocessor")) {}

    inline image preprocessor::preprocess(const std::string &input, const std::string &fallback_font) const {
        pugi::xml_document doc;
        if (const pugi::xml_parse_result result = doc.load_string(input.data()); !result) {
            logger->error("Failed to parse XML document: {}", result.description());
            return {input, {}};
        }

        std::string default_font = fallback_font;

        if (pugi::xml_node svg_node = doc.select_node("//*[local-name()='svg']").node(); !trim(svg_node.attribute("font-family").as_string()).empty()) {
            default_font = trim(svg_node.attribute("font-family").as_string());
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
