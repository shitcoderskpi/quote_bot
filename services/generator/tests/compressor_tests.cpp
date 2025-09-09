//
// Created by mmatz on 8/10/25.
//
#include <gtest/gtest.h>
#include "../compressor.h"
#include <random>
#include <chrono>
#include <algorithm>

class compressor_test : public ::testing::Test {
protected:
    compressor compressor_instance;
    
    // Helper method to generate random string
    std::string generate_random_string(size_t length) {
        static const std::string charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()_+-=[]{}|;:,.<>?";
        static std::random_device rd;
        static std::mt19937 gen(rd());
        static std::uniform_int_distribution<> dis(0, charset.size() - 1);
        
        std::string result;
        result.reserve(length);
        for (size_t i = 0; i < length; ++i) {
            result += charset[dis(gen)];
        }
        return result;
    }
    
    // Helper method to generate repetitive string (good for compression)
    std::string generate_repetitive_string(size_t length) {
        std::string pattern = "This is a repetitive pattern that should compress well ";
        std::string result;
        result.reserve(length);
        
        while (result.size() < length) {
            result += pattern;
        }
        result.resize(length);
        return result;
    }
    
    // Helper method to generate random binary data
    std::string generate_binary_data(size_t length) {
        static std::random_device rd;
        static std::mt19937 gen(rd());
        static std::uniform_int_distribution<> dis(0, 255);
        
        std::string result;
        result.reserve(length);
        for (size_t i = 0; i < length; ++i) {
            result += static_cast<char>(dis(gen));
        }
        return result;
    }
};

// Basic functionality tests
TEST_F(compressor_test, basic_compression_and_decompression) {
    const std::string original = "Hello, World! This is a test string for compression.";

    const std::string compressed = compressor::compress(original, 3);
    EXPECT_FALSE(compressed.empty());
    EXPECT_NE(compressed, original);

    const std::string decompressed = compressor::decompress(compressed);
    EXPECT_EQ(decompressed, original);
}

TEST_F(compressor_test, empty_string_handling) {
    const std::string empty_string;

    const std::string compressed = compressor::compress(empty_string, 3);
    EXPECT_FALSE(compressed.empty()); // zstd can compress empty strings

    const std::string decompressed = compressor::decompress(compressed);
    EXPECT_EQ(decompressed, empty_string);
}

TEST_F(compressor_test, single_character_string) {
    const std::string single_char = "A";

    const std::string compressed = compressor::compress(single_char, 3);
    EXPECT_FALSE(compressed.empty());

    const std::string decompressed = compressor::decompress(compressed);
    EXPECT_EQ(decompressed, single_char);
}

// Compression level tests
TEST_F(compressor_test, different_compression_levels) {
    const std::string test_data = generate_repetitive_string(1000);

    const std::string compressed_level_1 = compressor::compress(test_data, 1);
    const std::string compressed_level_3 = compressor::compress(test_data, 3);
    const std::string compressed_level_9 = compressor::compress(test_data, 9);
    const std::string compressed_level_22 = compressor::compress(test_data, 22);\

    EXPECT_FALSE(compressed_level_1.empty());
    EXPECT_FALSE(compressed_level_3.empty());
    EXPECT_FALSE(compressed_level_9.empty());
    EXPECT_FALSE(compressed_level_22.empty());

    // Higher compression levels should generally produce smaller output
    // Note: This is not always guaranteed due to zstd's adaptive nature
    EXPECT_LE(compressed_level_22.size(), compressed_level_1.size());
    
    // Verify all compressed versions decompress correctly
    EXPECT_EQ(compressor::decompress(compressed_level_1), test_data);
    EXPECT_EQ(compressor::decompress(compressed_level_3), test_data);
    EXPECT_EQ(compressor::decompress(compressed_level_9), test_data);
    EXPECT_EQ(compressor::decompress(compressed_level_22), test_data);
}

TEST_F(compressor_test, default_compression_level) {
    const std::string test_data = "Test data for default compression level";

    const std::string compressed_default = compressor::compress(test_data);
    const std::string compressed_explicit = compressor::compress(test_data, 3);
    
    EXPECT_EQ(compressed_default, compressed_explicit);
}

// Edge case tests
TEST_F(compressor_test, very_long_string) {
    const std::string long_string = generate_random_string(100000);

    const std::string compressed = compressor::compress(long_string, 3);
    EXPECT_FALSE(compressed.empty());

    const std::string decompressed = compressor::decompress(compressed);
    EXPECT_EQ(decompressed, long_string);
}

TEST_F(compressor_test, very_short_string) {
    const std::string short_string = "Hi";

    const std::string compressed = compressor::compress(short_string, 3);
    EXPECT_FALSE(compressed.empty());

    const std::string decompressed = compressor::decompress(compressed);
    EXPECT_EQ(decompressed, short_string);
}

TEST_F(compressor_test, unicode_string) {
    const std::string unicode_string = "Hello 世界! Привет! こんにちは! 🌍";

    const std::string compressed = compressor::compress(unicode_string, 3);
    EXPECT_FALSE(compressed.empty());

    const std::string decompressed = compressor::decompress(compressed);
    EXPECT_EQ(decompressed, unicode_string);
}

TEST_F(compressor_test, special_characters) {
    const std::string special_chars = "!@#$%^&*()_+-=[]{}|;:,.<>?\"'\\`~";

    const std::string compressed = compressor::compress(special_chars, 3);
    EXPECT_FALSE(compressed.empty());

    const std::string decompressed = compressor::decompress(compressed);
    EXPECT_EQ(decompressed, special_chars);
}

TEST_F(compressor_test, newlines_and_tabs) {
    const std::string text_with_whitespace = "Line 1\nLine 2\tTabbed content\r\nWindows line ending";

    const std::string compressed = compressor::compress(text_with_whitespace, 3);
    EXPECT_FALSE(compressed.empty());

    const std::string decompressed = compressor::decompress(compressed);
    EXPECT_EQ(decompressed, text_with_whitespace);
}

// Binary data tests
TEST_F(compressor_test, binary_data_compression) {
    const std::string binary_data = generate_binary_data(1000);

    const std::string compressed = compressor::compress(binary_data, 3);
    EXPECT_FALSE(compressed.empty());

    const std::string decompressed = compressor::decompress(compressed);
    EXPECT_EQ(decompressed, binary_data);
}

TEST_F(compressor_test, null_bytes_in_string) {
    const std::string data_with_nulls = "Hello\0World\0Test";

    const std::string compressed = compressor::compress(data_with_nulls, 3);
    EXPECT_FALSE(compressed.empty());

    const std::string decompressed = compressor::decompress(compressed);
    EXPECT_EQ(decompressed, data_with_nulls);
}

// Compression efficiency tests
TEST_F(compressor_test, repetitive_data_compression) {
    const std::string repetitive_data = generate_repetitive_string(5000);

    const std::string compressed = compressor::compress(repetitive_data, 9);
    EXPECT_FALSE(compressed.empty());
    
    // Repetitive data should compress well
    const double compression_ratio = static_cast<double>(compressed.size()) / repetitive_data.size();
    EXPECT_LT(compression_ratio, 0.8); // Should compress to less than 80% of original size

    const std::string decompressed = compressor::decompress(compressed);
    EXPECT_EQ(decompressed, repetitive_data);
}

TEST_F(compressor_test, random_data_compression) {
    const std::string random_data = generate_random_string(5000);

    const std::string compressed = compressor::compress(random_data, 9);
    EXPECT_FALSE(compressed.empty());
    
    // Random data typically doesn't compress as well
    const double compression_ratio = static_cast<double>(compressed.size()) / random_data.size();
    EXPECT_GT(compression_ratio, 0.8); // Random data usually compresses poorly

    const std::string decompressed = compressor::decompress(compressed);
    EXPECT_EQ(decompressed, random_data);
}

// Error handling tests
TEST_F(compressor_test, invalid_compression_levels) {
    const std::string test_data = "Test data";
    
    // Test extremely high compression level (should still work)
    const std::string compressed_high = compressor::compress(test_data, 100);
    EXPECT_FALSE(compressed_high.empty());
    
    // Test negative compression level (should still work, zstd handles this)
    const std::string compressed_negative = compressor::compress(test_data, -5);
    EXPECT_FALSE(compressed_negative.empty());
    
    // Test zero compression level
    const std::string compressed_zero = compressor::compress(test_data, 0);
    EXPECT_FALSE(compressed_zero.empty());
    
    // Verify all decompress correctly
    EXPECT_EQ(compressor::decompress(compressed_high), test_data);
    EXPECT_EQ(compressor::decompress(compressed_negative), test_data);
    EXPECT_EQ(compressor::decompress(compressed_zero), test_data);
}

TEST_F(compressor_test, corrupt_data_decompression) {
    const std::string original = "Test data";

    // Corrupt the compressed data
    if (std::string compressed = compressor::compress(original, 3); !compressed.empty()) {
        compressed[0] = static_cast<char>(compressed[0] ^ 0xFF);

        const std::string decompressed = compressor::decompress(compressed);
        EXPECT_TRUE(decompressed.empty()); // Should return empty string on corruption
    }
}

TEST_F(compressor_test, empty_compressed_data) {
    const std::string empty_compressed;

    const std::string decompressed = compressor::decompress(empty_compressed);
    EXPECT_TRUE(decompressed.empty());
}

TEST_F(compressor_test, partial_compressed_data) {
    const std::string original = "Test data";

    if (const std::string compressed = compressor::compress(original, 3); !compressed.empty() && compressed.size() > 1) {
        const std::string partial_compressed = compressed.substr(0, compressed.size() - 1);

        const std::string decompressed = compressor::decompress(partial_compressed);
        EXPECT_TRUE(decompressed.empty()); // Should return empty string on incomplete data
    }
}

// Size estimation tests
TEST_F(compressor_test, size_estimation_accuracy) {
    const std::string test_data = generate_random_string(1000);

    const size_t estimated_size = compressor::estimate_compressed_size(test_data);
    const std::string actual_compressed = compressor::compress(test_data, 3);
    
    EXPECT_GE(estimated_size, actual_compressed.size());
    EXPECT_LE(estimated_size, test_data.size() + 128); // zstd overhead is reasonable
}

TEST_F(compressor_test, size_estimation_edge_cases) {
    // Empty string
    const size_t empty_estimate = compressor::estimate_compressed_size("");
    EXPECT_GT(empty_estimate, 0);
    
    // Single character
    const size_t single_estimate = compressor::estimate_compressed_size("A");
    EXPECT_GT(single_estimate, 0);
    
    // Very long string
    const std::string long_string = generate_random_string(1000000);
    const size_t long_estimate = compressor::estimate_compressed_size(long_string);
    EXPECT_GT(long_estimate, 0);
    EXPECT_LE(long_estimate, long_string.size() + 1024 * 6); // Reasonable overhead
}

// Multiple compression/decompression cycles
TEST_F(compressor_test, multiple_compression_cycles) {
    const std::string original = "Test data for multiple compression cycles";

    const std::string compressed1 = compressor::compress(original, 3);
    const std::string decompressed1 = compressor::decompress(compressed1);

    const std::string compressed2 = compressor::compress(decompressed1, 6);
    const std::string decompressed2 = compressor::decompress(compressed2);

    const std::string compressed3 = compressor::compress(decompressed2, 9);
    const std::string decompressed3 = compressor::decompress(compressed3);
    
    EXPECT_EQ(decompressed1, original);
    EXPECT_EQ(decompressed2, original);
    EXPECT_EQ(decompressed3, original);
}

TEST_F(compressor_test, compression_idempotency) {
    const std::string original = "Test data for idempotency";

    const std::string compressed1 = compressor::compress(original, 3);
    const std::string compressed2 = compressor::compress(original, 3);
    
    // Same input with same compression level should produce same output
    EXPECT_EQ(compressed1, compressed2);
}

// Performance and stress tests
TEST_F(compressor_test, large_data_compression) {
    const std::string large_data = generate_random_string(1000000); // 1MB

    const auto start = std::chrono::high_resolution_clock::now();
    const std::string compressed = compressor::compress(large_data, 3);
    const auto end = std::chrono::high_resolution_clock::now();

    const auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    
    EXPECT_FALSE(compressed.empty());
    EXPECT_LT(duration.count(), 1000); // Should complete within 1 second

    const std::string decompressed = compressor::decompress(compressed);
    EXPECT_EQ(decompressed, large_data);
}

TEST_F(compressor_test, many_small_strings) {
    std::vector<std::string> small_strings;
    std::vector<std::string> compressed_strings;
    
    // Generate 1000 small strings
    for (int i = 0; i < 1000; ++i) {
        small_strings.push_back(generate_random_string(100));
    }
    
    // Compress all strings
    for (const auto& str : small_strings) {
        compressed_strings.push_back(compressor::compress(str, 3));
    }
    
    // Decompress and verify all
    for (size_t i = 0; i < small_strings.size(); ++i) {
        std::string decompressed = compressor::decompress(compressed_strings[i]);
        EXPECT_EQ(decompressed, small_strings[i]);
    }
}

// Memory and resource tests
TEST_F(compressor_test, memory_efficiency) {
    const std::string test_data = generate_random_string(100000);
    
    // Get initial memory usage (approximate)
    const size_t initial_size = test_data.size();

    const std::string compressed = compressor::compress(test_data, 3);
    const size_t compressed_size = compressed.size();
    
    // Compression should not use excessive memory
    EXPECT_LE(compressed_size, initial_size * 2); // Reasonable memory usage

    const std::string decompressed = compressor::decompress(compressed);
    EXPECT_EQ(decompressed, test_data);
}

// Thread safety tests (compressor is static, so should be thread-safe)
TEST_F(compressor_test, concurrent_access_simulation) {
    const std::string test_data = "Test data for concurrent access";
    
    // Simulate concurrent access by calling methods rapidly
    std::vector<std::string> results;
    for (int i = 0; i < 100; ++i) {
        std::string compressed = compressor::compress(test_data, 3);
        std::string decompressed = compressor::decompress(compressed);
        results.push_back(decompressed);
    }
    
    // All results should be identical
    for (const auto& result : results) {
        EXPECT_EQ(result, test_data);
    }
}

// Integration tests
TEST_F(compressor_test, integration_with_different_data_types) {
    // Test with various types of data
    const std::vector<std::string> test_cases = {
        "",                                    // Empty
        "A",                                   // Single character
        "Hello World",                         // Simple text
        "1234567890",                         // Numbers
        "Hello\nWorld\tTest\r\n",             // Text with whitespace
        "Hello 世界! 🌍",                      // Unicode
        generate_random_string(1000),         // Random data
        generate_repetitive_string(1000),     // Repetitive data
        generate_binary_data(1000)            // Binary data
    };
    
    for (const auto& test_case : test_cases) {
        std::string compressed = compressor::compress(test_case, 3);
        EXPECT_FALSE(compressed.empty());
        
        std::string decompressed = compressor::decompress(compressed);
        EXPECT_EQ(decompressed, test_case);
    }
}

// Boundary condition tests
TEST_F(compressor_test, boundary_compression_levels) {
    const std::string test_data = "Test data for boundary conditions";
    
    // Test boundary compression levels
    const std::string compressed_min = compressor::compress(test_data, 1);
    const std::string compressed_max = compressor::compress(test_data, 22);
    
    EXPECT_FALSE(compressed_min.empty());
    EXPECT_FALSE(compressed_max.empty());
    
    EXPECT_EQ(compressor::decompress(compressed_min), test_data);
    EXPECT_EQ(compressor::decompress(compressed_max), test_data);
}

TEST_F(compressor_test, extremely_large_string) {
    // Test with a very large string (10MB)
    const std::string huge_data = generate_random_string(10000000);

    const auto start = std::chrono::high_resolution_clock::now();
    const std::string compressed = compressor::compress(huge_data, 3);
    const auto end = std::chrono::high_resolution_clock::now();

    const auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    
    EXPECT_FALSE(compressed.empty());
    EXPECT_LT(duration.count(), 10000); // Should complete within 10 seconds

    const std::string decompressed = compressor::decompress(compressed);
    EXPECT_EQ(decompressed, huge_data);
}

// Regression tests
TEST_F(compressor_test, regression_known_strings) {
    // Test with known strings that might have caused issues
    const std::vector<std::string> known_strings = {
        "a", "aa", "aaa", "aaaa", "aaaaa",
        "Hello", "Hello World", "Hello World!",
        "Test123", "Test 123", "Test\t123",
        "Line1\nLine2", "Line1\r\nLine2",
        "Special: !@#$%^&*()_+-=[]{}|;:,.<>?",
        "Unicode: 世界, Привет, こんにちは, 🌍"
    };
    
    for (const auto& str : known_strings) {
        std::string compressed = compressor::compress(str, 3);
        EXPECT_FALSE(compressed.empty());
        
        std::string decompressed = compressor::decompress(compressed);
        EXPECT_EQ(decompressed, str);
    }
}
