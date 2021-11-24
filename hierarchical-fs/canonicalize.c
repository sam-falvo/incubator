#include <string.h>

#include "clifns.h"


#define DELIM(c) (((c) == '/') || ((c) == 0))


void
canonicalize(char *path) {
	char *pathend = path + strlen(path) + 1;
	char *ip, *op;

	ip = op = path;
	while(ip < pathend) {
		if(*ip == '/') {
			ip++;
			while((ip < pathend) && (*ip == '/')) ip++;
			*op++ = '/';
		}
		else if((ip <= pathend-2) && (ip[0] == '.') && DELIM(ip[1])) ip += 2;
		else if((ip <= pathend-3) && (ip[0] == '.') && (ip[1] == '.') && DELIM(ip[2])) {
			ascend(path, &op);
			ip += 3;
		}
		else *op++ = *ip++;
	}
	*op++ = 0;

	if(strlen(path) == 0) {
		strcpy(path, ".");
	}
}

