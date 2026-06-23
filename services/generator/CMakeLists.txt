cmake_minimum_required(VERSION 3.31)
project(generator)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_VERBOSE_MAKEFILE ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(CMAKE_CXX_FLAGS_RELEASE "${CMAKE_CXX_FLAGS} -mavx2 -O3 -march=native")
set(CMAKE_CXX_FLAGS_RELWITHDEBINFO "${CMAKE_CXX_FLAGS} -O2 -g -Wall")
set(CMAKE_CXX_FLAGS_DEBUG "${CMAKE_CXX_FLAGS} -O0 -g3 -fno-inline -fno-omit-frame-pointer -fno-strict-aliasing -Wall -Wextra -DDEBUG")

find_package(spdlog REQUIRED)
find_package(PkgConfig REQUIRED)
find_package(ImageMagick COMPONENTS Magick++ REQUIRED)
find_package(OpenSSL COMPONENTS Crypto REQUIRED)
find_package(OpenMP REQUIRED)
pkg_check_modules(ZSTD REQUIRED libzstd)

pkg_check_modules(HIREDIS REQUIRED hiredis)
pkg_check_modules(CAIRO REQUIRED cairo)
pkg_check_modules(PANGO REQUIRED pango)
pkg_check_modules(PANGOCAIRO REQUIRED pangocairo)

enable_testing()

include(FetchContent)
FetchContent_Declare(
        googletest
        URL https://github.com/google/googletest/archive/refs/heads/main.zip
)
FetchContent_MakeAvailable(googletest)

# Core (named `kernel` because of prometheus-cpp's core lib conflicting)
add_library(core
        globals.h
        redis_queue.cpp
        redis_queue.h
        redis_reply.cpp
        redis_reply.h
        image.cpp
        image.h
        text.cpp
        text.h
        text_factory.cpp
        text_factory.h
        renderer.cpp
        renderer.h
        rasterizer.cpp
        rasterizer.h
        response.cpp
        response.h
        compressor.cpp
        compressor.h
        exceptions.h
)
target_compile_definitions(core PRIVATE MAGICKCORE_HDRI_ENABLE=1)
target_include_directories(core PRIVATE ${HIREDIS_INCLUDE_DIRS})
target_include_directories(core PRIVATE ${ZSTD_INCLUDE_DIRS})
target_include_directories(core PRIVATE ${ImageMagick_INCLUDE_DIRS})
target_compile_definitions(core PRIVATE $<$<CONFIG:Debug>:DEBUG>)
target_include_directories(core PRIVATE
        ${CAIRO_INCLUDE_DIRS}
        ${PANGO_INCLUDE_DIRS}
        ${PANGOCAIRO_INCLUDE_DIRS}
)
link_directories(${PANGO_LIBRARY_DIRS})
target_link_libraries(core
        spdlog::spdlog
        ${HIREDIS_LINK_LIBRARIES}
        ${ImageMagick_LIBRARIES}
        OpenSSL::Crypto
        ${CAIRO_LIBRARIES}
        ${PANGO_LIBRARIES}
        ${PANGOCAIRO_LIBRARIES}
        OpenMP::OpenMP_CXX
        ${ZSTD_LIBRARIES}
)

add_definitions(
        ${CAIRO_CFLAGS_OTHER}
        ${PANGO_CFLAGS_OTHER}
        ${PANGOCAIRO_CFLAGS_OTHER}
        -DMAGICKCORE_QUANTUM_DEPTH=16
)

# Executable
add_executable(${PROJECT_NAME} main.cpp)
target_compile_definitions(${PROJECT_NAME} PRIVATE MAGICKCORE_HDRI_ENABLE=1)
target_include_directories(${PROJECT_NAME} PRIVATE ${HIREDIS_INCLUDE_DIRS})
target_include_directories(${PROJECT_NAME} PRIVATE ${ImageMagick_INCLUDE_DIRS})
target_include_directories(${PROJECT_NAME} PRIVATE
        ${CAIRO_INCLUDE_DIRS}
        ${PANGO_INCLUDE_DIRS}
        ${PANGOCAIRO_INCLUDE_DIRS}
)
link_directories(${PANGO_LIBRARY_DIRS})


target_link_libraries(${PROJECT_NAME} PRIVATE
        core
)

# Tests (using googletest)
add_executable(test_project tests/redis_tests.cpp
        tests/preproccessor_tests.cpp
        tests/request_json_parser_tests.cpp
        tests/compressor_tests.cpp
        tests/encoder_tests.cpp
        tests/redis_compressor_tests.cpp)

target_compile_definitions(test_project PRIVATE MAGICKCORE_HDRI_ENABLE=1)
target_include_directories(test_project PRIVATE ${HIREDIS_INCLUDE_DIRS})
target_include_directories(test_project PRIVATE ${ImageMagick_INCLUDE_DIRS})
target_include_directories(test_project PRIVATE
        ${CAIRO_INCLUDE_DIRS}
        ${PANGO_INCLUDE_DIRS}
        ${PANGOCAIRO_INCLUDE_DIRS}
)
link_directories(${PANGO_LIBRARY_DIRS})
target_link_libraries(test_project PRIVATE
        core
        GTest::gtest_main
)
