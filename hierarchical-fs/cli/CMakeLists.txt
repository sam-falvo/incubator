add_library(
	clilib STATIC
	"ascend.c"
	"canonicalize.c"
	"do_cd.c"
	"do_pwd.c"
	"evalcl.c"
	"skipws.c"
	"stripcmd.c"
)

add_executable(cli "cli.c")
target_link_libraries(cli clilib)

add_executable(t-stripcmd "t-stripcmd.c")
target_link_libraries(t-stripcmd clilib)
add_test(t-stripcmd t-stripcmd)

add_executable(t-skipws "t-skipws.c")
target_link_libraries(t-skipws clilib)
add_test(t-skipws t-skipws)

add_executable(t-canonicalize "t-canonicalize.c")
target_link_libraries(t-canonicalize clilib)
add_test(t-canonicalize t-canonicalize)

add_executable(t-ascend "t-ascend.c")
target_link_libraries(t-ascend clilib)
add_test(t-ascend t-ascend)

