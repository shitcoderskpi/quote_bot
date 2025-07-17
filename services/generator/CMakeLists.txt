cmake_minimum_required(VERSION 3.31)
project(generator)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_VERBOSE_MAKEFILE ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

find_package(spdlog REQUIRED)
find_package(PkgConfig REQUIRED)
find_package(ImageMagick COMPONENTS Magick++ REQUIRED)
find_package(OpenSSL COMPONENTS Crypto REQUIRED)
pkg_check_modules(HIREDIS REQUIRED hiredis)
add_subdirectory(extern/prometheus-cpp)
add_subdirectory(extern/cppcoro)



add_executable(${PROJECT_NAME} main.cpp
        global.h
        redis_queue.cpp
        redis_queue.h
        redis_reply.cpp
        redis_reply.h
)

target_compile_definitions(${PROJECT_NAME} PRIVATE MAGICKCORE_HDRI_ENABLE=1)

target_include_directories(${PROJECT_NAME} PRIVATE ${HIREDIS_INCLUDE_DIRS})
target_include_directories(${PROJECT_NAME} PRIVATE ${ImageMagick_INCLUDE_DIRS})
target_include_directories(${PROJECT_NAME} PRIVATE extern/prometheus-cpp/core/include})
target_include_directories(${PROJECT_NAME} PRIVATE extern/cppcoro/include)
target_link_libraries(${PROJECT_NAME} PRIVATE
        spdlog::spdlog
        ${HIREDIS_LINK_LIBRARIES}
        prometheus-cpp::core
        prometheus-cpp::pull
        cppcoro
        ${ImageMagick_LIBRARIES}
        OpenSSL::Crypto
)
