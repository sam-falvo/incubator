#include <unistd.h>
#include <stdbool.h>
#include <string.h>

#include <stdio.h>

#include "clifns.h"


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

	banner();
	while(!done) {
		length = readcl(cmdbuffer, CMDBUF_LEN);
		done = evalcl(cmdbuffer, length);
		// Printing happens as a consequence of executing commands.
	}

	return 0;
}

