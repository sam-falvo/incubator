#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <string.h>

#include "clifns.h"
#include "dm.h"


bool
evalcl(DmAnchorBlock *ab, char *buf, size_t len) {
	bool exitrequested = false;
	char *saveptr;
	char *delim = "\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F\x10"
	              "\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F ";
	char *cmd, *args;

	cmd = strtok_r(buf, delim, &saveptr);
	args = buf + strlen(cmd) + 1;
	skipws(&args);

	if(!strcmp(buf, "exit")) exitrequested = true;
	else if(!strcmp(buf, "pwd")) do_pwd(ab, args);
	else if(!strcmp(buf, "cd")) do_cd(ab, args);
	else {
		printf("%s: Command not supported\n", cmd);
	}

	return exitrequested;
}


