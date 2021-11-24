#include <stdbool.h>
#include <string.h>

#include "clifns.h"


#define DELIM(c) (((c) == '/') || ((c) == 0))


static bool
validelement(char *p) {
	return (
		(*p != '.' && !DELIM(*p))
	||	(p[0] == '.' && p[1] == '.' && !DELIM(p[2]))
	);
}


void
canonicalize(char *path) {
	char *pathend = path + strlen(path);
	char *ip, *op;

	ip = op = path;
	while(ip < pathend) {
		if(*ip == '/') {
			ip++;
			while((ip < pathend) && (*ip == '/')) ip++;
			*op++ = '/';
		}
		else if(validelement(ip)) {
			while(!DELIM(*ip)) *op++ = *ip++;
		}
		else if((ip <= pathend-1) && (ip[0] == '.') && DELIM(ip[1])) ip += 2;
		else if((ip <= pathend-2) && (ip[0] == '.') && (ip[1] == '.') && DELIM(ip[2])) {
			ascend(path, &op);
			ip += 3;
		}
		else *op++ = *ip++;
	}

	// Handle the case of /a/b/c/ -> /a/b/c, but without sacrificing
	// the case where the canonical path is just /.
	if((path < (op-1)) && (op[-1] == '/')) op--;

	*op++ = 0;

	if(strlen(path) == 0) {
		strcpy(path, ".");
	}
}

