#include <stdio.h>
#include <string.h>

#include "dm.h"


void
do_cd(DmAnchorBlock *ab, char *args) {
	int erc;

	erc = DmChangeDir(ab, args);
	if(erc != E_OK) {
		printf("%s: path not found (error %d)\n", args, erc);
	}
}

