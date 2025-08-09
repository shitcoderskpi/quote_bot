//
// Created by mmatz on 8/9/25.
//

#ifndef JSON_PARSER_H
#define JSON_PARSER_H
#include <string>
#include <simdjson.h>
#include <spdlog/spdlog.h>

#include "globals.h"

// ABC
template<typename T>
class json_parser {
    public:
    json_parser() noexcept = default;
    virtual ~json_parser() noexcept = default;

    virtual T parse(const std::string&) = 0;

};


struct request {
    long long chat_id;
    std::string svg_template;
};

// Impl
class request_json_parser final : public json_parser<request> {
    public:
    request_json_parser() noexcept : logger(logger_init("request_json_parser")) {}
    ~request_json_parser() noexcept override = default;

    request parse(const std::string&) override;
private:
    simdjson::ondemand::parser parser;
    const std::shared_ptr<spdlog::logger> logger;
};

#endif //JSON_PARSER_H
