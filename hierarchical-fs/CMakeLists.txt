project(hierarchical-fs)
cmake_minimum_required(VERSION 3.18)

add_executable(cli "cli.c")

enable_testing()

add_executable(cli-test "cli-test.c")
add_test(cli-test cli-test)

add_executable(t-stripcmp "t-stripcmd.c")
add_test(t-stripcmd t-stripcmd)

