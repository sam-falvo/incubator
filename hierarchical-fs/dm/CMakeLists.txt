add_library(
	dmlib STATIC
	"DmChangeDir.c"
	"DmCurrentDir.c"
	"DmExpunge.c"
	"DmInitialize.c"
)
target_include_directories(dmlib PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}")

