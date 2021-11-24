#include <stdio.h>
#include <string.h>


void
do_cd(char *args) {
	if(!strcmp(args, "/")) return;
	printf("%s: path not found\n", args);
}


