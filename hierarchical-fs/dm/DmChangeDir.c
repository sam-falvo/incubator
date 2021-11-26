#include <string.h>

#include <dm.h>


int
DmChangeDir(DmAnchorBlock *ab, char *path) {
	if(!strcmp(path, "/")) return E_OK;
	return E_NOTEXISTS;
}

