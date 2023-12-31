#include <stdbool.h>
#include <string.h>

#include "slice.h"


slice_t
slice_init(slice_t s) {
	if(s) {
		s->start = s->end = -1;
	}
	return s;
}

bool
slice_range_eq(slice_t s, int start, int end) {
	if(!s) return false;
	return (s->start == start) && (s->end == end);
}

bool
slice_range_ne(slice_t s, int start, int end) {
	if(!s) return false;
	return !slice_range_eq(s, start, end);
}

bool
slice_string_eq(slice_t s, section_t sect, char *str) {
	int len1 = slice_length(s);
	int len2 = strlen(str);

	if(len1 != len2) return false;

	return section_memcmp_eq(sect, s->start, (uint8_t *)str, len1);
}

bool
slice_string_ne(slice_t s, section_t sect, char *str) {
	int len1 = slice_length(s);
	int len2 = strlen(str);

	if(len1 != len2) return false;

	return section_memcmp_ne(sect, s->start, (uint8_t *)str, len1);
}

size_t
slice_length(slice_t s) {
	return s->end - s->start;
}
