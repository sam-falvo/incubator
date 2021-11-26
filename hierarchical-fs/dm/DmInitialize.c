#include <stdlib.h>
#include <string.h>

#include <dm.h>


#define DFLT_CURDIR_LEN		32


DmAnchorBlock *
DmInitialize(void) {
	DmAnchorBlock *ab;
	char *curdir;

	curdir = malloc(DFLT_CURDIR_LEN);
	if(!curdir) return NULL;
	strcpy(curdir, "/");

	ab = malloc(sizeof(DmAnchorBlock));
	if(ab) {
		memset(ab, 0, sizeof(DmAnchorBlock));

		ab->curdirlen = DFLT_CURDIR_LEN;
		ab->curdir = curdir;
	}
	return ab;
}

