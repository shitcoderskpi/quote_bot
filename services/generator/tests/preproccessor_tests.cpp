//
// Created by mmatz on 8/8/25.
//
#include <gtest/gtest.h>
#include <fstream>
#include <sstream>
#include <string>
#include <vector>

#include "../preprocessor.h"
#include "../text.h"

std::string read_file(const std::string &filename) {
    std::ifstream svg {filename, std::ios::in};
    if (!svg) return  "";

    std::stringstream ss;
    ss << svg.rdbuf();

    return ss.str();
}

const auto svg = read_file("../templates/tg_template.svg");

TEST(preproccessor_tests, preprocess_test) {
    templates::preprocessor preprocessor;
    
    // Test with valid SVG containing text elements
    std::string valid_svg = R"(
        <svg xmlns="http://www.w3.org/2000/svg" width="360" height="110" font-family="Arial">
            <text x="56" y="45" font-size="13px" fill="#000">Hello World</text>
            <text x="56" y="23" font-size="14px" font-weight="bold" fill="#2a5885">John Doe</text>
        </svg>
    )";
    
    auto result = preprocessor.preprocess(valid_svg, "sans-serif");
    
    // Verify background contains SVG without text elements
    EXPECT_FALSE(result.background.empty());
    EXPECT_EQ(result.background.find("<text"), std::string::npos);
    
    // Verify text entries were extracted
    EXPECT_EQ(result.text_entries.size(), 2);
    
    // Check first text entry
    EXPECT_EQ(result.text_entries[0].content, "Hello World");
    EXPECT_EQ(result.text_entries[0].x, 56);
    EXPECT_EQ(result.text_entries[0].y, 45);
    EXPECT_EQ(result.text_entries[0].font_family, "Arial");
    EXPECT_EQ(result.text_entries[0].size, "13px");
    EXPECT_EQ(result.text_entries[0].color, "#000");
    EXPECT_EQ(result.text_entries[0].weight, pango::normal);
    
    // Check second text entry
    EXPECT_EQ(result.text_entries[1].content, "John Doe");
    EXPECT_EQ(result.text_entries[1].x, 56);
    EXPECT_EQ(result.text_entries[1].y, 23);
    EXPECT_EQ(result.text_entries[1].font_family, "Arial");
    EXPECT_EQ(result.text_entries[1].size, "14px");
    EXPECT_EQ(result.text_entries[1].color, "#2a5885");
    EXPECT_EQ(result.text_entries[1].weight, pango::bold);
}

TEST(preproccessor_tests, preprocess_empty_svg_test) {
    const templates::preprocessor preprocessor;
    
    // Test with SVG containing no text elements
    const std::string empty_svg = R"(
        <svg xmlns="http://www.w3.org/2000/svg" width="360" height="110">
            <rect x="10" y="10" width="100" height="50" fill="red"/>
            <circle cx="50" cy="50" r="20" fill="blue"/>
        </svg>
    )";
    
    auto [background, text_entries] = preprocessor.preprocess(empty_svg, "sans-serif");
    
    EXPECT_FALSE(background.empty());
    EXPECT_EQ(text_entries.size(), 0);
    EXPECT_EQ(background.find("<text"), std::string::npos);
}

TEST(preproccessor_tests, preprocess_invalid_xml_test) {
    const templates::preprocessor preprocessor;
    
    // Test with invalid XML
    const std::string invalid_xml = "<svg><text>Hello</text><unclosed>";
    
    auto [background, text_entries] = preprocessor.preprocess(invalid_xml, "sans-serif");
    
    // Should return original input as background and empty text entries
    EXPECT_EQ(background, invalid_xml);
    EXPECT_EQ(text_entries.size(), 0);
}

TEST(preproccessor_tests, preprocess_font_family_override_test) {
    const templates::preprocessor preprocessor;
    
    // Test SVG with font-family attribute on svg element
    const std::string svg_with_font = R"(
        <svg xmlns="http://www.w3.org/2000/svg" width="360" height="110" font-family="Times New Roman">
            <text x="56" y="45" font-size="13px" fill="#000">Test Text</text>
        </svg>
    )";
    
    auto [background, text_entries] = preprocessor.preprocess(svg_with_font, "sans-serif");
    
    EXPECT_EQ(text_entries.size(), 1);
    EXPECT_EQ(text_entries[0].font_family, "Times New Roman");
    EXPECT_EQ(text_entries[0].content, "Test Text");
}

TEST(preproccessor_tests, preprocess_text_alignment_test) {
    const templates::preprocessor preprocessor;
    
    // Test text with different alignments
    const std::string svg_with_alignment = R"(
        <svg xmlns="http://www.w3.org/2000/svg" width="360" height="110">
            <text x="56" y="45" font-size="13px" fill="#000" text-anchor="middle">Center Text</text>
            <text x="56" y="65" font-size="13px" fill="#000" text-anchor="end">Right Text</text>
            <text x="56" y="85" font-size="13px" fill="#000">Left Text</text>
        </svg>
    )";
    
    auto [background, text_entries] = preprocessor.preprocess(svg_with_alignment, "sans-serif");
    
    EXPECT_EQ(text_entries.size(), 3);
    EXPECT_EQ(text_entries[0].alignment, PANGO_ALIGN_CENTER);
    EXPECT_EQ(text_entries[1].alignment, PANGO_ALIGN_RIGHT);
    EXPECT_EQ(text_entries[2].alignment, PANGO_ALIGN_LEFT);
}

TEST(preproccessor_tests, preprocess_font_weight_test) {
    const templates::preprocessor preprocessor;
    
    // Test text with different font weights
    const std::string svg_with_weights = R"(
        <svg xmlns="http://www.w3.org/2000/svg" width="360" height="110">
            <text x="56" y="25" font-size="13px" font-weight="light" fill="#000">Light Text</text>
            <text x="56" y="45" font-size="13px" font-weight="normal" fill="#000">Normal Text</text>
            <text x="56" y="65" font-size="13px" font-weight="bold" fill="#000">Bold Text</text>
            <text x="56" y="85" font-size="13px" font-weight="bolder" fill="#000">Bolder Text</text>
        </svg>
    )";
    
    auto [background, text_entries] = preprocessor.preprocess(svg_with_weights, "sans-serif");
    
    EXPECT_EQ(text_entries.size(), 4);
    EXPECT_EQ(text_entries[0].weight, pango::light);
    EXPECT_EQ(text_entries[1].weight, pango::normal);
    EXPECT_EQ(text_entries[2].weight, pango::bold);
    EXPECT_EQ(text_entries[3].weight, pango::ultrabold);
}

TEST(preproccessor_tests, preprocess_stress_test) {
    templates::preprocessor preprocessor;
    
    // Generate a large SVG with many text elements
    std::stringstream large_svg;
    large_svg << R"(<svg xmlns="http://www.w3.org/2000/svg" width="1000" height="10000">)";

    int n = 100'000;
    // Add `n` text elements
    for (int i = 0; i < n; ++i) {
        large_svg << "<text x=\"" << (i % 50) * 20 << "\" y=\"" << (i / 50) * 20 + 20 
                  << R"(" font-size="12px" fill="#000">Text )" << i << "</text>";
    }
    large_svg << "</svg>";
    
    auto [background, text_entries] = preprocessor.preprocess(large_svg.str(), "sans-serif");
    
    // Verify all text elements were processed
    EXPECT_EQ(text_entries.size(), n);
    EXPECT_FALSE(background.empty());
    
    // Verify no text elements remain in background
    EXPECT_EQ(background.find("<text"), std::string::npos);
    
    // Verify some specific text entries
    EXPECT_EQ(text_entries[0].content, "Text 0");
    EXPECT_EQ(text_entries[n - 1].content, "Text " + std::to_string(n - 1));
    EXPECT_EQ(text_entries[0].x, 0);
    EXPECT_EQ(text_entries[0].y, 20);
}

TEST(preproccessor_tests, preprocess_complex_svg_test) {
    // Test with the actual template file
    if (!svg.empty()) {
        const templates::preprocessor preprocessor;
        auto [background, text_entries] = preprocessor.preprocess(svg, "sans-serif");
        
        // Verify text elements were extracted
        EXPECT_GT(text_entries.size(), 0);
        EXPECT_FALSE(background.empty());
        
        // Verify no text elements remain in background
        EXPECT_EQ(background.find("<text"), std::string::npos);
        
        // Check that background still contains other SVG elements
        EXPECT_NE(background.find("<svg"), std::string::npos);
        EXPECT_NE(background.find("<defs"), std::string::npos);
        EXPECT_NE(background.find("<circle"), std::string::npos);
        EXPECT_NE(background.find("<rect"), std::string::npos);
    }
}

TEST(preproccessor_tests, preprocess_malformed_text_test) {
    const templates::preprocessor preprocessor;
    
    // Test with text elements missing attributes
    const std::string malformed_svg = R"(
        <svg xmlns="http://www.w3.org/2000/svg" width="360" height="110">
            <text>Text without attributes</text>
            <text x="56">Text with partial attributes</text>
            <text y="45">Text with partial attributes</text>
        </svg>
    )";
    
    auto [background, text_entries] = preprocessor.preprocess(malformed_svg, "sans-serif");
    
    // Should still process the text elements with default values
    EXPECT_EQ(text_entries.size(), 3);
    EXPECT_EQ(text_entries[0].content, "Text without attributes");
    EXPECT_EQ(text_entries[1].content, "Text with partial attributes");
    EXPECT_EQ(text_entries[2].content, "Text with partial attributes");
}

TEST(preproccessor_tests, preprocess_nested_elements_test) {
    const templates::preprocessor preprocessor;
    
    // Test with nested SVG elements
    const std::string nested_svg = R"(
        <svg xmlns="http://www.w3.org/2000/svg" width="360" height="110">
            <g>
                <text x="56" y="45" font-size="13px" fill="#000">Nested Text 1</text>
                <g>
                    <text x="56" y="65" font-size="13px" fill="#000">Nested Text 2</text>
                </g>
            </g>
            <text x="56" y="85" font-size="13px" fill="#000">Direct Text</text>
        </svg>
    )";
    
    auto [background, text_entries] = preprocessor.preprocess(nested_svg, "sans-serif");
    
    EXPECT_EQ(text_entries.size(), 3);
    EXPECT_EQ(text_entries[0].content, "Nested Text 1");
    EXPECT_EQ(text_entries[1].content, "Nested Text 2");
    EXPECT_EQ(text_entries[2].content, "Direct Text");
}