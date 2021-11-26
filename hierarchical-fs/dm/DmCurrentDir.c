#include <string.h>

#include <dm.h>

#include <stdio.h>

int
DmCurrentDir(DmAnchorBlock *ab, char *path, int *len) {
	int minpathlen = strlen(ab->curdir) + 1;

	if(*len < ab->curdirlen) {
		*len = ab->curdirlen;
		return E_TOOSMALL;
	}

	strcpy(path, ab->curdir);
	return E_OK;
}

