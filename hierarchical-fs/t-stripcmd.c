#include <stdio.h>
#include <string.h>

#include "clifns.h"

int
main(int argc, char *argv[]) {
	static char *vecs[] = {
		"hello world\r\n",
		"hello\tworld\r\n",
		"hello\nworld to yo too",
	};
	static char *exps[] = {
		"hello world",
		"hello\tworld",
		"hello",
	};

	char buffer[128];
	int i;

	for(i = 0; i < 3; i++) {
		strncpy(buffer, vecs[i], 128);
		buffer[127] = 0;

		stripcmd(buffer, strlen(buffer));

		if(strcmp(buffer, exps[i])) {
			fprintf(stderr, "Failed case %d\n", i);
			return 1;
		}
	}

	return 0;
}

