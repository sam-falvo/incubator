#pragma once

#include "slice.h"
#include "section.h"

typedef struct reader_desc *reader_t;

struct reader_desc {
	slice_t   slice;
	section_t section;
	int       index;
};

reader_t reader_init(reader_t, slice_t, section_t);
int reader_peek_char(reader_t);
void reader_next_char(reader_t);

