#ifndef CLIFNS_H
#define CLIFNS_H

#include <stddef.h>
#include <stdbool.h>

#include "dm.h"


void ascend(char *path, char **pop);
void canonicalize(char *path);
bool evalcl(DmAnchorBlock *ab, char *cmdstr, size_t length);
void skipws(char **ps);
void stripcmd(char *buffer, size_t length);

void do_pwd(DmAnchorBlock *, char *);
void do_cd(DmAnchorBlock *, char *);

#endif

