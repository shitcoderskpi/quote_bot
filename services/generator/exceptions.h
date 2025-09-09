//
// Created by mmatz on 8/24/25.
//

#ifndef EXCEPTIONS_H
#define EXCEPTIONS_H
#include <stdexcept>

namespace exceptions {
    namespace redis {
        class mismatched_type_error final : public std::runtime_error {
        public:
            mismatched_type_error() : std::runtime_error("Mismatched type.") {}
            explicit mismatched_type_error(const std::string &_arg) : std::runtime_error(_arg) {}
            explicit mismatched_type_error(const char *string) : std::runtime_error(string) {}
            explicit mismatched_type_error(std::runtime_error &&runtime_error) : std::runtime_error(runtime_error) {}
            explicit mismatched_type_error(const std::runtime_error &runtime_error) :
                std::runtime_error(runtime_error) {}
        };

        class empty_context_error final : public std::runtime_error {
        public:
            empty_context_error() : std::runtime_error("Context is empty.") {}
            explicit empty_context_error(const std::string &_arg) : std::runtime_error(_arg) {}
            explicit empty_context_error(const char *string) : std::runtime_error(string) {}
            explicit empty_context_error(std::runtime_error &&runtime_error) : std::runtime_error(runtime_error) {}
            explicit empty_context_error(const std::runtime_error &runtime_error) : std::runtime_error(runtime_error) {}
        };
    } // namespace redis

    namespace templates {
        class preprocessor_error final : public std::runtime_error {
        public:
            explicit preprocessor_error(const std::string &_arg) : std::runtime_error(_arg) {}
            explicit preprocessor_error(const char *string) : std::runtime_error(string) {}
            explicit preprocessor_error(std::runtime_error &&runtime_error) : std::runtime_error(runtime_error) {}
            explicit preprocessor_error(const std::runtime_error &runtime_error) : std::runtime_error(runtime_error) {}
        };
    } // namespace templates
} // namespace exceptions

#endif // EXCEPTIONS_H
