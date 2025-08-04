cmake_minimum_required(VERSION 3.31)
project(generator)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_VERBOSE_MAKEFILE ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

find_package(spdlog REQUIRED)
find_package(PkgConfig REQUIRED)
find_package(ImageMagick COMPONENTS Magick++ REQUIRED)
find_package(OpenSSL COMPONENTS Crypto REQUIRED)
find_library(PUGIXML_LIBRARY NAMES pugixml)
pkg_check_modules(HIREDIS REQUIRED hiredis)

cmake_policy(SET CMP0002 NEW)
set(ENABLE_TESTING OFF CACHE BOOL "Disable tests in prometheus-cpp" FORCE)
add_subdirectory(extern/prometheus-cpp EXCLUDE_FROM_ALL)

enable_testing()

include(FetchContent)
FetchContent_Declare(
        googletest
        URL https://github.com/google/googletest/archive/refs/heads/main.zip
)
FetchContent_MakeAvailable(googletest)

add_subdirectory(extern/cppcoro)

# Core (named `kernel` because of prometheus-cpp's core lib conflicting)
add_library(kernel
        globals.h
        redis_queue.cpp
        redis_queue.h
        redis_reply.cpp
        redis_reply.h
        storage.h
        storage.cpp
        preprocessor.cpp
        preprocessor.h
        image.cpp
        image.h
        text.cpp
        text.h
        text_factory.cpp
        text_factory.h
        renderer.cpp
        renderer.h
)
target_compile_definitions(kernel PRIVATE MAGICKCORE_HDRI_ENABLE=1)
target_include_directories(kernel PRIVATE ${HIREDIS_INCLUDE_DIRS})
target_include_directories(kernel PRIVATE ${ImageMagick_INCLUDE_DIRS})
target_include_directories(kernel PRIVATE extern/prometheus-cpp/core/include})
target_include_directories(kernel PRIVATE extern/cppcoro/include)
target_include_directories(kernel PRIVATE ${PUGIXML_INCLUDE_DIR})
target_link_libraries(kernel
        spdlog::spdlog
        ${HIREDIS_LINK_LIBRARIES}
        prometheus-cpp::core
        prometheus-cpp::pull
        cppcoro
        ${ImageMagick_LIBRARIES}
        OpenSSL::Crypto
        ${PUGIXML_LIBRARY}
)

# Executable
add_executable(${PROJECT_NAME} main.cpp)
target_compile_definitions(${PROJECT_NAME} PRIVATE MAGICKCORE_HDRI_ENABLE=1)
target_include_directories(${PROJECT_NAME} PRIVATE ${HIREDIS_INCLUDE_DIRS})
target_include_directories(${PROJECT_NAME} PRIVATE ${ImageMagick_INCLUDE_DIRS})
target_include_directories(${PROJECT_NAME} PRIVATE extern/prometheus-cpp/core/include})
target_include_directories(${PROJECT_NAME} PRIVATE extern/cppcoro/include)
target_link_libraries(${PROJECT_NAME} PRIVATE
        kernel
)

# Tests (using googletest)
add_executable(test_project tests/redis_tests.cpp
        tests/template_storage_tests.cpp
        tests/base64_tests.cpp)
target_compile_definitions(test_project PRIVATE MAGICKCORE_HDRI_ENABLE=1)
target_include_directories(test_project PRIVATE ${HIREDIS_INCLUDE_DIRS})
target_include_directories(test_project PRIVATE ${ImageMagick_INCLUDE_DIRS})
target_include_directories(test_project PRIVATE extern/prometheus-cpp/core/include})
target_include_directories(test_project PRIVATE extern/cppcoro/include)
target_link_libraries(test_project PRIVATE
        kernel
        GTest::gtest_main
)
