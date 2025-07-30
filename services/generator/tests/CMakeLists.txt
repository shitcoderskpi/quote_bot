cmake_minimum_required(VERSION 3.31)
project(tests)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_VERBOSE_MAKEFILE ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)


include(FetchContent)
FetchContent_Declare(
        googletest
        URL https://github.com/google/googletest/archive/03597a01ee50ed33e9dfd640b249b4be3799d395.zip
)
FetchContent_MakeAvailable(googletest)

add_executable(${PROJECT_NAME}
        redis_tests.cpp
        template_storage_tests.cpp
        globals.h
        base64_tests.cpp
)

target_link_libraries(${PROJECT_NAME} PRIVATE
        GTest::gtest_main
        cppcoro
)

include(GoogleTest)
gtest_discover_tests(${PROJECT_NAME})