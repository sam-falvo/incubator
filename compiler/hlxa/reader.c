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
