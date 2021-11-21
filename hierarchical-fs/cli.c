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
	actual = read(STDIN_FILENO, buf, len);
	buf[len-1] = 0;
	stripcmd(buf, actual);
	return actual;
}


static bool
evalcl(char *buf, size_t len) {
	bool exitrequested = false;
	char *saveptr;
	char *delim = "\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F"
	              "\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F ";
	char *cmd;

	cmd = strtok_r(buf, delim, &saveptr);

	if(!strcmp(buf, "exit")) exitrequested = true;
	else {
		write(STDOUT_FILENO, cmd, strlen(cmd));
		puts(": Command not supported\n");
	}

	return exitrequested;
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

