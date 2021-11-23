#ifndef CLIFNS_H
#define CLIFNS_H

#include <stddef.h>
#include <stdbool.h>

bool evalcl(char *cmdstr, size_t length);
void skipws(char **ps);
void stripcmd(char *buffer, size_t length);

void do_pwd(char *);
void do_cd(char *);

#endif

