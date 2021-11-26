#include <stdlib.h>

#include <dm.h>


void
DmExpunge(DmAnchorBlock *ab) {
	if(ab->curdir) free(ab->curdir);
	free(ab);
}

