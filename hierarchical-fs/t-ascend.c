#include <stdio.h>
#include <string.h>

#include "clifns.h"


typedef struct {
	char *vec;
	int opin;
	int opout;
} VEC;

int
main(int argc, char *argv[]) {
	static VEC vecs[] = {
		// path          opin   opout
		{ "/..",         1,     1, },
		{ "..",          0,     0, },
		{ "foo/bar/..",  8,     4, },
		{ "foo/bar/..",  4,     0, },
		{ "/foo/bar/..", 5,     1, },
	};
	char buffer[128];
	char *op, *opexp, *opstart;
	int i;
	int nvecs = sizeof(vecs) / sizeof(VEC);

	for(i = 0; i < nvecs; i++) {
		strncpy(buffer, vecs[i].vec, 128);
		buffer[127] = 0;
		op = opstart = buffer + vecs[i].opin;
		opexp = buffer + vecs[i].opout;

		ascend(buffer, &op);

		if(op != opexp) {
			fprintf(stderr, "Failed case: %d\n", i);
			fprintf(stderr, "     Vector: %s , %p\n", vecs[i].vec, opstart);
			fprintf(stderr, "   Expected: %p\n", opexp);
			fprintf(stderr, "     Actual: %p\n", op);
			return 1;
		}
	}

	return 0;
}

