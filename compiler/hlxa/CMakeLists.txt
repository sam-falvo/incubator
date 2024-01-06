cmake_minimum_required(VERSION 3.0)

project(hlxa C)

add_executable(test test.c section.c assembler.c statement.c slice.c dc_context.c reader.c)
# add_compile_definitions(OLD)
