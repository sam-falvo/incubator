#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>

#include "section.h"
#include "hlxa.h"

static hlxa_t hlxa_alloc(void);
static hlxa_t hlxa_init(hlxa_t);

struct hlxa_desc {
	section_t current_section;   // Section into which we're currently assembling
};

static hlxa_t
hlxa_alloc(void) {
	return (hlxa_t)malloc(sizeof(struct hlxa_desc));
}

static hlxa_t
hlxa_init(hlxa_t a) {
	if(a) {
		memset(a, 0, sizeof(struct hlxa_desc));
	}
	return a;
}

hlxa_t
hlxa_new(void) {
	return hlxa_init(hlxa_alloc());
}

void
hlxa_free(hlxa_t *pa) {
	if(pa && *pa) {
		free(*pa);
		*pa = NULL;
	}
}

// Sets the section into which the assembler is generating code.
void
hlxa_set_section(hlxa_t a, section_t s) {
	a->current_section = s;
}

// Attempts to assemble a single source line.
void
hlxa_assemble_line(hlxa_t a, char *linebuf) {
	section_append_byte(a->current_section, 0x01);
}
