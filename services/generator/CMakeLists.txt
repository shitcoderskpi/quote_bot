cmake_minimum_required(VERSION 3.31)
project(generator)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_VERBOSE_MAKEFILE ON)

find_package(spdlog REQUIRED)
find_package(PkgConfig REQUIRED)
pkg_check_modules(HIREDIS REQUIRED hiredis)
add_subdirectory(extern/prometheus-cpp)


add_executable(${PROJECT_NAME} main.cpp)

target_include_directories(${PROJECT_NAME} PRIVATE ${HIREDIS_INCLUDE_DIRS})
target_include_directories(${PROJECT_NAME} PRIVATE extern/prometheus-cpp/core/include})
target_link_libraries(${PROJECT_NAME} PRIVATE
        spdlog::spdlog
        ${HIREDIS_LINK_LIBRARIES}
        prometheus-cpp::core
        prometheus-cpp::pull
)
