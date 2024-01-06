#include <string.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <ctype.h>

#include "section.h"

#define MIN(a,b)                   (((a)<(b))?(a):(b))
#define DEFAULT_SECTION_CAPACITY   256

static section_t section_alloc(void);
static section_t section_init(section_t);

struct section_desc {
	uint8_t *buffer;
	size_t length;
	size_t capacity;
};

section_t
section_new(void) {
	return section_init(section_alloc());
}

section_t
section_new_from_string(char *s) {
	section_t sect = section_new();
	section_append_string(sect, s);
	return sect;
}

void
section_free(section_t *ps) {
	if(ps && *ps) {
		if((*ps)->buffer) free((*ps)->buffer);
		free(*ps);
		*ps = NULL;
	}
}

// If the section lacks a buffer, create one and reset length to zero.
// Otherwise, leave the buffer as-is.  If the buffer failed to allocate,
// leave s->buffer set to NULL.
static void
section_guarantee_buffer(section_t s) {
	if(s && !s->buffer) {
		s->buffer = (uint8_t *)malloc(DEFAULT_SECTION_CAPACITY);
		s->length = 0;
		s->capacity = DEFAULT_SECTION_CAPACITY;
	}
}

// Append a byte to the indicated section buffer.
//
// This MAY involve a buffer reallocation if there's not enough space.
// If a buffer hasn't yet been allocated, one will be allocated.
//
// If there is no memory for this operation, the buffer will be reset to
// NULL.
void
section_append_byte(section_t s, uint8_t byte) {
	section_guarantee_buffer(s);
	if(s->length < s->capacity) {
		if(s->buffer) {
			s->buffer[s->length] = byte;
			++ s->length;
		}
	}
	// How to handle out-of-memory case here?
}

// Compare the contents of two buffers for equality.
//
// If both sections are NULL, then by definition, their "contents"
// (which is to say, the mathematical concept of bottom) are equal.
//
// If one section is NULL and the other is not, then by definition
// their contents cannot be equal.
//
// Only if both sections are non-null and have valid buffers and
// those buffers have contents which are identical is the result
// true.
bool
section_compare_eq(section_t s1, section_t s2) {
	if(!s1) {
		if(!s2) return true;           // (NULL, NULL)
		if(!s2->buffer) return true;   // (NULL, s2.buffer==NULL)
		return false;                  // (NULL, s2.buffer)
	}
	if(!s2) {
		if(!s1->buffer) return true;   // (s1.buffer==NULL, NULL)
    return false;                  // (s1.buffer, NULL)
	}

	// Answer false if buffer lengths don't exactly match;
	// if they differ, then the contents necessarily must also differ.
	if(s1->length != s2->length) return false;

	// We now know s1 and s2 are non-NULL, that their buffer pointers
	// refer to valid memory, AND they have equal lengths.  All that
	// remains now is to compare each byte in those buffers.
	return 0 == memcmp(s1->buffer, s2->buffer, s1->length);
}

static section_t
section_alloc(void) {
	return (section_t)malloc(sizeof(struct section_desc));
}

static section_t
section_init(section_t s) {
	if(s) {
		memset(s, 0, sizeof(struct section_desc));
	}
	return s;
}

void
section_append_string(section_t s, char *str) {
	// Slow, but correct.
	while(*str) {
		section_append_byte(s, (int)*str++);
	}
}

int
section_byte_at(section_t s, int at) {
	if(at < s->length) {
		return s->buffer[at];
	}

	return -1;
}

size_t
section_length(section_t s) {
	return s->length;
}

bool
section_memcmp_eq(section_t s, int start, uint8_t *buf, size_t length) {
	int end = start + length;

	if(start >= s->length) return false;
	if(end   >  s->length) return false;

	return 0 == memcmp(&s->buffer[start], buf, length);
}

bool
section_memcmp_ne(section_t s, int start, uint8_t *buf, size_t length) {
	int end = start + length;

	if(start >= s->length) return false;
	if(end   >  s->length) return false;

	return 0 != memcmp(&s->buffer[start], buf, length);
}

char *
section_byte_address_fixme(section_t s, int start) {
	return (char *)&s->buffer[start];
}

static void
trim_trailing_whitespace(section_t s) {
	size_t length = s->length;

	while((length > 0) && isspace(s->buffer[length-1]))
		--length;

	s->length = length;
	s->buffer[length] = 0;
}

bool
section_refill_from_file(section_t s, FILE *fp) {
	char *str;

	section_guarantee_buffer(s);
	str = fgets((char *)s->buffer, s->capacity, fp);
	s->buffer[s->capacity - 1] = 0;
	s->length = strlen((char *)s->buffer);
	trim_trailing_whitespace(s);
	return str != NULL;
}

void
section_debug_print_buffer(FILE *fp, section_t s) {
	fprintf(fp, "%s\n", s->buffer);
}

uint8_t *
section_borrow_buffer(section_t s) {
	return s->buffer;
}

