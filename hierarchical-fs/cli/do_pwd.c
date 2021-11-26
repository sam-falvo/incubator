#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include "dm.h"


static char *pathbuf = NULL;
static int pathlen = 0;

void
do_pwd(DmAnchorBlock *ab, char *args) {
	int erc;

	do{
		erc = DmCurrentDir(ab, pathbuf, &pathlen);

		if(erc == E_TOOSMALL) {
			if(pathbuf) free(pathbuf);
			pathbuf = malloc(pathlen);
			if(!pathbuf) erc = E_NOMEM;
		}
	} while(erc == E_TOOSMALL);

	if(erc != E_OK) {
		fprintf(stderr, "DmCurrentDir failed: %d\n", erc);
		return;
	}

	printf("%s\n", pathbuf);
}

