//
// Created by mmatz on 8/9/25.
//

#include "json_parser.h"

request request_json_parser::parse(const std::string &json) {
    request result {};

    try {
        const simdjson::padded_string padded_json{json};
        auto doc = parser.iterate(padded_json);

        result.chat_id = doc["chat_id"].get_int64().value();
        result.svg_template = std::string{doc["template"].get_string().value()};
    } catch (const simdjson::simdjson_error &e) {
        logger->error("Parse error: {}", e.what());
    }

    return result;
}
