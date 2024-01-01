#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>
#include <ctype.h>

#include "section.h"
#include "hlxa.h"
#include "dc_context.h"

static hlxa_t hlxa_alloc(void);
static hlxa_t hlxa_init(hlxa_t);

struct hlxa_desc {
	section_t current_section;   // Section into which we're currently assembling
	int       errors;
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
is_hexdigit(uint8_t ch) {
	return (
		((ch >= '0') && (ch <= '9')) ||
		((ch >= 'A') && (ch <= 'F')) ||
		((ch >= 'a') && (ch <= 'f'))
	);
}

#ifdef OLD

// Attempts to assemble a single source line.
void
hlxa_assemble_statement(hlxa_t a, section_t sect, statement_t s) {
	char *linebuf, *p;
	int byte = 0;
	slice_t operand_slice = statement_borrow_operand(s);

	if(slice_length(operand_slice) == 0) {
		a->errors |= ERRF_MISSING_OPERAND;
		return;
	}

	if(slice_string_ne(statement_borrow_mnemonic(s), sect, "DC")) {
		a->errors |= ERRF_UNKNOWN_MNEMONIC;
		return;
	}

	// The remaining code should belong in a Reader abstraction of some kind.

	linebuf = section_byte_address_fixme(sect, operand_slice->start);
	p = linebuf + 2; // skip over initial X'

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

#else // ==============================================================

// Attempts to assemble a single source line.
void
hlxa_assemble_statement(hlxa_t a, section_t inp, statement_t s) {
	slice_t operand_slice = statement_borrow_operand(s);
	struct dc_context_desc context;

	// Currently, we only recognize the DC mnemonic.

	if(slice_string_ne(statement_borrow_mnemonic(s), inp, "DC")) {
		a->errors |= ERRF_UNKNOWN_MNEMONIC;
		return;
	}

	// DC must have at least one operand.

	if(slice_length(operand_slice) == 0) {
		a->errors |= ERRF_MISSING_OPERAND;
		return;
	}

	dc_context_init(&context);
	dc_context_decode(operand_slice, inp, &context);
	dc_context_validate(&context, inp);
	if(dc_context_errors(&context)) {
		// How to log specific context errors?
		a->errors |= ERRF_BAD_OPERAND;
		return;
	}

	// If here, context is valid; assemble the bytes into the current section.
}

#endif

// Answers with the current set of errors.
int
hlxa_errors(hlxa_t a) {
	return a->errors;
}
