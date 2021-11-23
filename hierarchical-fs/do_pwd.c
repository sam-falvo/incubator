#include <unistd.h>


void
do_pwd(char *args) {
	write(STDOUT_FILENO, "/\n", 2);
}

