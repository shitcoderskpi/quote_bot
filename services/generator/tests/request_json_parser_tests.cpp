//
// Created by mmatz on 8/9/25.
//
#include <gtest/gtest.h>
#include "../json_parser.h"

class request_json_parser_test : public ::testing::Test {
protected:
    request_json_parser parser;
};

TEST_F(request_json_parser_test, valid_json_parsing) {
    const std::string valid_json = R"({
        "chat_id": 123456789,
        "template": "<svg>test template</svg>"
    })";

    const auto [chat_id, svg_template] = parser.parse(valid_json);

    EXPECT_EQ(chat_id, 123456789);
    EXPECT_EQ(svg_template, "<svg>test template</svg>");
}

TEST_F(request_json_parser_test, different_chat_id_values) {
    const std::string json_zero = R"({
        "chat_id": 0,
        "template": "test"
    })";
    const auto [chat_id, svg_template] = parser.parse(json_zero);
    EXPECT_EQ(chat_id, 0);

    const std::string json_negative = R"({
        "chat_id": -123456,
        "template": "test"
    })";
    const request result_negative = parser.parse(json_negative);
    EXPECT_EQ(result_negative.chat_id, -123456);

    const std::string json_large = R"({
        "chat_id": 9223372036854775807,
        "template": "test"
    })";
    const request result_large = parser.parse(json_large);
    EXPECT_EQ(result_large.chat_id, 9223372036854775807);
}

TEST_F(request_json_parser_test, different_template_content) {
    const std::string json_empty = R"({
        "chat_id": 123,
        "template": ""
    })";
    const auto [chat_id, svg_template] = parser.parse(json_empty);
    EXPECT_EQ(svg_template, "");

    const std::string special_chars = R"(<svg><text>&quot;&amp;&lt;&gt;</text></svg>)";
    const std::string json_special = R"({
        "chat_id": 123,
        "template": ")" + special_chars + R"("
    })";
    const request result_special = parser.parse(json_special);
    EXPECT_EQ(result_special.svg_template, special_chars);
}

TEST_F(request_json_parser_test, whitespace_variations) {
    const std::string json_minimal = R"({"chat_id":123,"template":"test"})";
    const request result_minimal = parser.parse(json_minimal);
    EXPECT_EQ(result_minimal.chat_id, 123);
    EXPECT_EQ(result_minimal.svg_template, "test");

    const std::string json_extra_whitespace = R"({
        "chat_id" : 123,
        "template" : "test"
    })";
    const request result_extra = parser.parse(json_extra_whitespace);
    EXPECT_EQ(result_extra.chat_id, 123);
    EXPECT_EQ(result_extra.svg_template, "test");
}

TEST_F(request_json_parser_test, invalid_json_handling) {
    const std::string malformed_json = R"({
        "chat_id": 123,
        "template": "test"
    )";

    const request result_malformed = parser.parse(malformed_json);
    EXPECT_EQ(result_malformed.chat_id, 0);
    EXPECT_EQ(result_malformed.svg_template, "");

    const std::string empty_json;
    const request result_empty = parser.parse(empty_json);
    EXPECT_EQ(result_empty.chat_id, 0);
    EXPECT_EQ(result_empty.svg_template, "");

    const std::string non_json = "This is not JSON";
    const request result_non_json = parser.parse(non_json);
    EXPECT_EQ(result_non_json.chat_id, 0);
    EXPECT_EQ(result_non_json.svg_template, "");
}

TEST_F(request_json_parser_test, missing_fields) {
    const std::string missing_chat_id = R"({
        "template": "test"
    })";
    const request result_missing_chat_id = parser.parse(missing_chat_id);
    EXPECT_EQ(result_missing_chat_id.chat_id, 0);
    EXPECT_EQ(result_missing_chat_id.svg_template, "");

    const std::string missing_template = R"({
        "chat_id": 123
    })";
    const request result_missing_template = parser.parse(missing_template);
    EXPECT_EQ(result_missing_template.chat_id, 123);
    EXPECT_EQ(result_missing_template.svg_template, "");

    const std::string empty_object = R"({})";
    const request result_empty_object = parser.parse(empty_object);
    EXPECT_EQ(result_empty_object.chat_id, 0);
    EXPECT_EQ(result_empty_object.svg_template, "");
}

TEST_F(request_json_parser_test, wrong_field_types) {
    const std::string chat_id_string = R"({
        "chat_id": "123",
        "template": "test"
    })";
    const request result_chat_id_string = parser.parse(chat_id_string);
    EXPECT_EQ(result_chat_id_string.chat_id, 0);
    EXPECT_EQ(result_chat_id_string.svg_template, "");

    const std::string template_number = R"({
        "chat_id": 123,
        "template": 456
    })";
    const request result_template_number = parser.parse(template_number);
    EXPECT_EQ(result_template_number.chat_id, 123);
    EXPECT_EQ(result_template_number.svg_template, "");

    const std::string chat_id_bool = R"({
        "chat_id": true,
        "template": "test"
    })";
    request result_chat_id_bool = parser.parse(chat_id_bool);
    EXPECT_EQ(result_chat_id_bool.chat_id, 0);
    EXPECT_EQ(result_chat_id_bool.svg_template, "");
}

TEST_F(request_json_parser_test, extra_fields) {
    const std::string extra_fields = R"({
        "chat_id": 123,
        "template": "test",
        "extra_field": "should be ignored",
        "another_field": 456
    })";

    const request result = parser.parse(extra_fields);
    EXPECT_EQ(result.chat_id, 123);
    EXPECT_EQ(result.svg_template, "test");
}

TEST_F(request_json_parser_test, field_order_independence) {
    const std::string template_first = R"({
        "template": "test",
        "chat_id": 123
    })";

    const request result = parser.parse(template_first);
    EXPECT_EQ(result.chat_id, 123);
    EXPECT_EQ(result.svg_template, "test");
}

TEST_F(request_json_parser_test, null_values) {
    const std::string null_chat_id = R"({
        "chat_id": null,
        "template": "test"
    })";
    const request result_null_chat_id = parser.parse(null_chat_id);
    EXPECT_EQ(result_null_chat_id.chat_id, 0);
    EXPECT_EQ(result_null_chat_id.svg_template, "");

    const std::string null_template = R"({
        "chat_id": 123,
        "template": null
    })";
    const request result_null_template = parser.parse(null_template);
    EXPECT_EQ(result_null_template.chat_id, 123);
    EXPECT_EQ(result_null_template.svg_template, "");
}

TEST_F(request_json_parser_test, nested_objects) {
    const std::string nested_chat_id = R"({
        "chat_id": {"nested": "object"},
        "template": "test"
    })";
    const request result_nested_chat_id = parser.parse(nested_chat_id);
    EXPECT_EQ(result_nested_chat_id.chat_id, 0);
    EXPECT_EQ(result_nested_chat_id.svg_template, "");

    const std::string nested_template = R"({
        "chat_id": 123,
        "template": {"nested": "object"}
    })";
    const request result_nested_template = parser.parse(nested_template);
    EXPECT_EQ(result_nested_template.chat_id, 123);
    EXPECT_EQ(result_nested_template.svg_template, "");
}

TEST_F(request_json_parser_test, array_values) {
    const std::string array_chat_id = R"({
        "chat_id": [1, 2, 3],
        "template": "test"
    })";
    const request result_array_chat_id = parser.parse(array_chat_id);
    EXPECT_EQ(result_array_chat_id.chat_id, 0);
    EXPECT_EQ(result_array_chat_id.svg_template, "");

    const std::string array_template = R"({
        "chat_id": 123,
        "template": ["a", "b", "c"]
    })";
    request result_array_template = parser.parse(array_template);
    EXPECT_EQ(result_array_template.chat_id, 123);
    EXPECT_EQ(result_array_template.svg_template, "");
}

TEST_F(request_json_parser_test, very_large_numbers) {
    const std::string large_number = R"({
        "chat_id": 9223372036854775808,
        "template": "test"
    })";
    const request result_large = parser.parse(large_number);
    EXPECT_EQ(result_large.chat_id, 0);
    EXPECT_EQ(result_large.svg_template, "");

    const std::string small_number = R"({
        "chat_id": -9223372036854775809,
        "template": "test"
    })";
    const request result_small = parser.parse(small_number);
    EXPECT_EQ(result_small.chat_id, 0);
    EXPECT_EQ(result_small.svg_template, "");
}

TEST_F(request_json_parser_test, very_long_template) {
    const std::string long_template(10000, 'x');
    const std::string json_long = R"({
        "chat_id": 123,
        "template": ")" + long_template + R"("
    })";

    const request result = parser.parse(json_long);
    EXPECT_EQ(result.chat_id, 123);
    EXPECT_EQ(result.svg_template, long_template);
}

TEST_F(request_json_parser_test, unicode_template) {
    const std::string unicode_template = "Hello 世界 🌍";
    const std::string json_unicode = R"({
        "chat_id": 123,
        "template": ")" + unicode_template + R"("
    })";

    const request result = parser.parse(json_unicode);
    EXPECT_EQ(result.chat_id, 123);
    EXPECT_EQ(result.svg_template, unicode_template);
}

TEST_F(request_json_parser_test, multiple_parse_calls) {
    request_json_parser parser_instance;

    const std::string json1 = R"({
        "chat_id": 123,
        "template": "first"
    })";

    const std::string json2 = R"({
        "chat_id": 456,
        "template": "second"
    })";

    const request result1 = parser_instance.parse(json1);
    const request result2 = parser_instance.parse(json2);

    EXPECT_EQ(result1.chat_id, 123);
    EXPECT_EQ(result1.svg_template, "first");
    EXPECT_EQ(result2.chat_id, 456);
    EXPECT_EQ(result2.svg_template, "second");
}

TEST_F(request_json_parser_test, floating_point_numbers) {
    const std::string float_chat_id = R"({
        "chat_id": 123.456,
        "template": "test"
    })";
    const request result_float = parser.parse(float_chat_id);
    EXPECT_EQ(result_float.chat_id, 0);
    EXPECT_EQ(result_float.svg_template, "");
}

TEST_F(request_json_parser_test, boolean_values) {
    const std::string bool_chat_id = R"({
        "chat_id": false,
        "template": "test"
    })";
    const request result_bool = parser.parse(bool_chat_id);
    EXPECT_EQ(result_bool.chat_id, 0);
    EXPECT_EQ(result_bool.svg_template, "");
}

TEST_F(request_json_parser_test, empty_template_content) {
    const std::string json_empty_content = R"({
        "chat_id": 123,
        "template": ""
    })";

    const request result = parser.parse(json_empty_content);
    EXPECT_EQ(result.chat_id, 123);
    EXPECT_EQ(result.svg_template, "");
}

TEST_F(request_json_parser_test, whitespace_only_template) {
    const std::string json_whitespace = R"({
        "chat_id": 123,
        "template": "   \t\n\r   "
    })";

    const request result = parser.parse(json_whitespace);
    EXPECT_EQ(result.chat_id, 123);
    EXPECT_EQ(result.svg_template, "   \t\n\r   ");
}