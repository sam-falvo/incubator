#include "clifns.h"


void
ascend(char *path, char **pop) {
	char *fence;
	char *op;

	fence = path;
	if(path[0] == '/') fence++;

	op = *pop;
	if(op[-1] == '/') op--;
	if(op < fence) op = fence;

	while((op > fence) && (op[-1] != '/')) op--;

	*pop = op;
}

