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
int      reader_peek_char(reader_t);
void     reader_next_char(reader_t);
int      reader_read_integer(reader_t);
void     reader_subslice_string(reader_t, slice_t);
int      reader_read_byte_hex(reader_t);
