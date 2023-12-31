#pragma once

#include "statement.h"

typedef struct hlxa_desc *hlxa_t;

#define ERRF_MISSING_OPERAND   0x0001
#define ERRF_UNKNOWN_MNEMONIC  0x0002

hlxa_t hlxa_new(void);
void   hlxa_free(hlxa_t *);
void   hlxa_set_section(hlxa_t, section_t);
void   hlxa_assemble_statement(hlxa_t, section_t, statement_t);
int    hlxa_errors(hlxa_t);
