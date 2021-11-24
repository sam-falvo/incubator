#include <stdio.h>
#include <string.h>

#include "clifns.h"


typedef struct {
	char *vec;
	char *exp;
} VEC;

int
main(int argc, char *argv[]) {
	static VEC vecs[] = {
		// input                  expected
		{ "//",                   "/",         },
		{ "////",                 "/",         },
		{ "./",                   ".",         },
		{ "/.",                   "/",         },
		{ "a/b/c",                "a/b/c",     },
		{ "/a/b/c",               "/a/b/c",    },
		{ "a/b/c/",               "a/b/c/",    },	// not minimal, but not bad either.
		{ "/a/b/c/",              "/a/b/c/"    },	// not minimal, but not bad either.
		{ "/dev/./cons",          "/dev/cons", },
		{ ".",                    ".",         },
		{ "..",                   ".",         },
		{ "/..",                  "/",         },
		{ "foo/bar/../baz",       "foo/baz",   },
		{ "foo/bar/../../baz",    "baz",       },
		{ "foo/bar/../../../baz", "baz",       },
		{ "./../path",            "path",      },
		{ "foo/./bar/./../baz",   "foo/baz",   },
	};
	char buffer[128];
	char *pbuf;
	int i;
	int nvecs = sizeof(vecs) / sizeof(VEC);

	for(i = 0; i < nvecs; i++) {
		strncpy(buffer, vecs[i].vec, 128);
		buffer[127] = 0;

		canonicalize(buffer);

		if(strcmp(buffer, vecs[i].exp)) {
			fprintf(stderr, "Failed case: %d\n", i);
			fprintf(stderr, "     Vector: %s\n", vecs[i].vec);
			fprintf(stderr, "   Expected: %s\n", vecs[i].exp);
			fprintf(stderr, "     Actual: %s\n", buffer);
			return 1;
		}
	}

	return 0;
}

