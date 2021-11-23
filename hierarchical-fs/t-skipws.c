#include <stdio.h>
#include <string.h>

#include "clifns.h"

int
main(int argc, char *argv[]) {
	static char *vecs[] = {
		"hello world",
		"hello\tworld",
		"             hello\tworld",
		"\t\thello\tworld",
		"\r\n       \t\t",
	};
	static char *exps[] = {
		"hello world",
		"hello\tworld",
		"hello\tworld",
		"hello\tworld",
		"",
	};

	char buffer[128];
	char *pbuf;
	int i;

	for(i = 0; i < 5; i++) {
		strncpy(buffer, vecs[i], 128);
		buffer[127] = 0;
		pbuf = buffer;

		skipws(&pbuf);

		if(strcmp(pbuf, exps[i])) {
			fprintf(stderr, "Failed case %d\n", i);
			return 1;
		}
	}

	return 0;
}

