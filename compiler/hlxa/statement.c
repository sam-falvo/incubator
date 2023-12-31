#include <stdlib.h>
#include <string.h>

#include "slice.h"
#include "section.h"
#include "statement.h"


struct statement_desc {
};


static statement_t
statement_alloc(void) {
	return (statement_t)malloc(sizeof(struct statement_desc));
}

static statement_t
statement_init(statement_t s) {
  if(s) {
		memset(s, -1, sizeof(struct statement_desc));
	}
	return s;
}

statement_t
statement_new(void) {
	return statement_init(statement_alloc());
}

void
statement_free(statement_t *ps) {
	if(ps && *ps) {
		free(*ps);
		*ps = NULL;
	}
}

void
statement_decode(section_t linebuf, statement_t out) {
}

int
statement_errors(statement_t s) {
	return 0;
}

slice_t
statement_borrow_label(statement_t s) {
	return NULL;
}

slice_t
statement_borrow_mnemonic(statement_t s) {
	return NULL;
}

slice_t
statement_borrow_operand(statement_t s) {
	return NULL;
}
