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

// Convert a hex value into a binary value
static int
hex_value(char ch) {
	int i = ch;

	// Assume ASCII.
	i -= 0x30;
	if(i > 0x09) { // it was either an A-F or a-f
		i -= 7;
		if(i > 0x0F) { // it was a lowercase a-f
			i -= 0x20;
		}
	}
	return i;
}

// answers true iff the digit is a hexadecimal digit
static bool
is_hexdigit(char ch) {
	return (
			((ch >= '0') && (ch <= '9')) ||
			((ch >= 'A') && (ch <= 'F')) ||
			((ch >= 'a') && (ch <= 'f'))
	);
}

// Attempts to assemble a single source line.
void
hlxa_assemble_line(hlxa_t a, char *linebuf) {
	char *p = linebuf + 2; // skip over initial X'
	int byte = 0;

	// Accumulate a byte
	while(is_hexdigit(*p)) {
		byte = (byte << 4) | hex_value(*p);
		++ p;

		if(is_hexdigit(*p)) {
			byte = (byte << 4) | hex_value(*p);
			++ p;
		}

		section_append_byte(a->current_section, byte);
		byte = 0;
	}
}
