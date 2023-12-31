#include <string.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>

#include "section.h"

static section_t section_alloc(void);
static section_t section_init(section_t);

struct section_desc {
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

void
section_append_byte(section_t s, uint8_t byte) {
}

bool
section_compare_eq(section_t s1, section_t s2) {
	return false;
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

