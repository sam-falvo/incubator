#include <string.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>

#include "section.h"

#define MIN(a,b)                   (((a)<(b))?(a):(b))
#define DEFAULT_SECTION_CAPACITY   256

static section_t section_alloc(void);
static section_t section_init(section_t);

struct section_desc {
	uint8_t *buffer;
	size_t length;
};

section_t
section_new(void) {
	return section_init(section_alloc());
}

void
section_free(section_t *ps) {
	if(ps && *ps) {
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
	if(s->buffer) {
		s->buffer[s->length] = byte;
		++ s->length;
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

