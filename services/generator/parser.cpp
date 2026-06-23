#include "parser.h"
#include <charconv>
#include <expected>
#include <format>

namespace {
    std::string_view read_until(std::string_view& str, char delim) {
        const auto pos = str.find(delim);
        if (pos == std::string_view::npos) {
            const auto res = str;
            str = {};
            return res;
        }
        const auto res = str.substr(0, pos);
        str.remove_prefix(pos + 1);
        return res;
    }

    std::expected<int, std::string> parse_int(const std::string_view str) {
        int val = 0;
        if (auto [ptr, ec] = std::from_chars(str.data(), str.data() + str.size(), val); ec != std::errc{}) {
            return std::unexpected("Failed to parse integer");
        }
        return val;
    }
}

std::expected<parsed_message, std::string> parser::parse(std::string_view payload) {
    parsed_message result;

    while (!payload.empty()) {
        auto type_str = read_until(payload, ';');
        if (type_str.empty()) continue;

        auto len_str = read_until(payload, ';');
        if (len_str.empty()) break;

        const auto len = parse_int(len_str);
        if (!len) {
            return std::unexpected(std::format("Failed to parse message: {}", len.error()));
        }

        if (payload.size() < static_cast<std::size_t>(*len)) {
            return std::unexpected("Payload size is less than expected length");
        }

        std::string_view data = payload.substr(0, *len);
        payload.remove_prefix(*len);

        // skip the trailing comma if it exists
        if (!payload.empty() && payload.front() == ',') {
            payload.remove_prefix(1);
        }

        const auto type = parse_int(type_str);
        if (!type) {
            return std::unexpected {std::format("Failed to parse message: {}", type.error())};
        }

        switch (*type) {
            case 0: {
                result.svg = {std::string(data)};
            } break;
            case 1: {
                pango_message pango;
                pango.x = parse_int(read_until(data, ';')).value();
                pango.y = parse_int(read_until(data, ';')).value();
                pango.wrap_width = parse_int(read_until(data, ';')).value();
                pango.wrap_mode = [&data] constexpr {
                    switch (parse_int(read_until(data, ';')).value()) {
                        case 1: return PANGO_WRAP_CHAR;
                        case 2: return PANGO_WRAP_WORD_CHAR;
                        default: break;
                    }
                    return PANGO_WRAP_WORD;
                }();
                pango.alignment = [&data] constexpr {
                    switch (parse_int(read_until(data, ';')).value()) {
                        case 1: return PANGO_ALIGN_CENTER;
                        case 2: return PANGO_ALIGN_RIGHT;
                        default: break;
                    }
                    return PANGO_ALIGN_LEFT;
                }();
                pango.font_description = std::string(read_until(data, ';'));
                pango.markup = std::string(data); // The rest is markup
                result.pangos.push_back(std::move(pango));
            } break;
            case 2: {
                result.metadata.chat_id = std::string(data);
            } break;
            default: break;
        }
    }

    return result;
}
