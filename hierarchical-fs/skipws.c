#include <unistd.h>
#include <ctype.h>

#include "clifns.h"


void
skipws(char **buffer) {
	while(isspace(**buffer)) (*buffer)++;
}
