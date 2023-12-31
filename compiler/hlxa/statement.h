#pragma once

#include "slice.h"
#include "section.h"

typedef struct statement_desc *statement_t;

statement_t statement_new(void);
void        statement_free(statement_t *);

void        statement_decode(section_t, statement_t);
int         statement_errors(statement_t);
slice_t     statement_borrow_label(statement_t);
slice_t     statement_borrow_mnemonic(statement_t);
slice_t     statement_borrow_operand(statement_t);
