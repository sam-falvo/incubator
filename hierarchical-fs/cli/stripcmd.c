#include <unistd.h>

#include "clifns.h"


void
stripcmd(char *buffer, size_t length) {
	int i;

	for(i = 0; i < length; i++) {
		char ch = buffer[i];
		if((0 <= ch) && (ch <= 31)) {
			if(ch == '\t') continue;
			ch = 0;
		}
		buffer[i] = ch;
	}
}
