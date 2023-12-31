#include <stdlib.h>
#include <string.h>
#include <ctype.h>

#include "slice.h"
#include "section.h"
#include "statement.h"


struct statement_desc {
	struct slice_desc label_slice;
	struct slice_desc mnemonic_slice;
	struct slice_desc operand_slice;
	int               errors;
};


static statement_t
statement_alloc(void) {
	return (statement_t)malloc(sizeof(struct statement_desc));
}

static statement_t
statement_init(statement_t s) {
  if(s) {
		memset(s, 0, sizeof(struct statement_desc));
		slice_init(&s->label_slice);
		slice_init(&s->mnemonic_slice);
		slice_init(&s->operand_slice);
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

static void
detect_field(section_t linebuf, size_t length, int *pstart, int *pend, slice_t slice) {
	int start = *pstart, end = *pend;

	if(start >= length) return;
	if(end >= length) return;

	if(!isspace(section_byte_at(linebuf, end))) {
		while(
			(end < length) &&
			!isspace(section_byte_at(linebuf, end))
		) ++end;

		slice->start = start;
		slice->end = end;

		*pstart = end;
		*pend = end;
	}
}

static void
skip_whitespace(section_t linebuf, size_t length, int *pstart, int *pend) {
	int start = *pstart;
	int end = *pend;

	if(start >= length) return;
	if(end >= length) return;

  while(
		(start < length) &&
		(isspace(section_byte_at(linebuf, start)))
	) ++ start;

	*pstart = start;
	*pend = start;
}

void
statement_decode(section_t linebuf, statement_t out) {
	int start = 0;
	int end = 0;
	size_t length = section_length(linebuf);

	detect_field(linebuf, length, &start, &end, statement_borrow_label(out));
	skip_whitespace(linebuf, length, &start, &end);
	detect_field(linebuf, length, &start, &end, statement_borrow_mnemonic(out));
	skip_whitespace(linebuf, length, &start, &end);
	detect_field(linebuf, length, &start, &end, statement_borrow_operand(out));
}

int
statement_errors(statement_t s) {
	return s->errors;
}

slice_t
statement_borrow_label(statement_t s) {
	return &s->label_slice;
}

slice_t
statement_borrow_mnemonic(statement_t s) {
	return &s->mnemonic_slice;
}

slice_t
statement_borrow_operand(statement_t s) {
	return &s->operand_slice;
}
