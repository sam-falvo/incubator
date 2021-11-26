#include <unistd.h>
#include <stdbool.h>
#include <string.h>

#include <stdio.h>

#include "clifns.h"

#include "dm.h"


#define CMDBUF_LEN 256


static void
banner(void) {
	puts("Hierarchical File Space Explorer CLI\n\n");
}


static size_t
readcl(char *buf, size_t len) {
	size_t actual;

	write(STDOUT_FILENO, "^ ", 2);
	memset(buf, 0, len);
	actual = read(STDIN_FILENO, buf, len-1);
	stripcmd(buf, actual);
	return actual;
}


int
main(int argc, char *argv[]) {
	bool done = false;
	static char cmdbuffer[CMDBUF_LEN];
	size_t length;
	DmAnchorBlock *ab;

	ab = DmInitialize();
	if(!ab) {
		fprintf(stderr, "DmInitialize() failed: out of memory\n");
		return 127;
	}

	banner();
	while(!done) {
		length = readcl(cmdbuffer, CMDBUF_LEN);
		done = evalcl(ab, cmdbuffer, length);
		// Printing happens as a consequence of executing commands.
	}

	DmExpunge(ab);

	return 0;
}

