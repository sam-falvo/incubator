#pragma once

#include "statement.h"

typedef struct assembler_desc *assembler_t;

#define ERRF_MISSING_OPERAND   0x0001
#define ERRF_UNKNOWN_MNEMONIC  0x0002
#define ERRF_BAD_OPERAND       0x0004

assembler_t assembler_new(void);
void        assembler_free(assembler_t *);
void        assembler_set_section(assembler_t, section_t);
void        assembler_assemble_statement(assembler_t, section_t, statement_t);
int         assembler_errors(assembler_t);
