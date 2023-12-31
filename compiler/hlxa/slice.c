#include <stdbool.h>

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

