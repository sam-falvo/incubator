#include <ctype.h>

#include "reader.h"


reader_t
reader_init(reader_t rd, slice_t sl, section_t sec) {
	if(rd) {
		rd->slice = sl;
		rd->section = sec;

		rd->index = sl->start;
	}
	return rd;
}


int
reader_peek_char(reader_t rd) {
	int ch = -1;
	if(rd->index < rd->slice->end)  ch = section_byte_at(rd->section, rd->index);
	return ch;
}


void
reader_next_char(reader_t rd) {
	if(rd->index < rd->slice->end)  ++ rd->index;
}


int
reader_read_integer(reader_t rd) {
	int value = 0;
	int ch;

	ch = reader_peek_char(rd);
	while(isdigit(ch)) {
		value = value * 10 + (ch - '0');
		reader_next_char(rd);
		ch = reader_peek_char(rd);
	}

	return value;
}


void
reader_subslice_string(reader_t rd, slice_t s) {
	int quote, ch;

	quote = reader_peek_char(rd);
	reader_next_char(rd);
	s->start = rd->index;
	s->end = rd->index;

	ch = reader_peek_char(rd);
	while((ch != quote) && (ch > 0)) {
		++ s->end;
		reader_next_char(rd);
		ch = reader_peek_char(rd);
	}
	
	if(ch == quote) reader_next_char(rd);
}
