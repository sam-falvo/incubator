project(hierarchical-fs)
cmake_minimum_required(VERSION 3.18)

enable_testing()

include_directories("include")

add_subdirectory("dm")
add_subdirectory("cli")
