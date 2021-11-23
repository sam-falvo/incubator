project(hierarchical-fs)
cmake_minimum_required(VERSION 3.18)

add_library(
	clilib STATIC
	"stripcmd.c"
	"skipws.c"
)

add_executable(cli "cli.c")
target_link_libraries(cli clilib)

enable_testing()

add_executable(t-stripcmd "t-stripcmd.c")
target_link_libraries(t-stripcmd clilib)
add_test(t-stripcmd t-stripcmd)

add_executable(t-skipws "t-skipws.c")
target_link_libraries(t-skipws clilib)
add_test(t-skipws t-skipws)

