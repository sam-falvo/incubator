#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>
#include <ctype.h>

#include "reader.h"
#include "section.h"
#include "assembler.h"
#include "dc_context.h"

static assembler_t assembler_alloc(void);
static assembler_t assembler_init(assembler_t);

struct assembler_desc {
	section_t current_section;   // Section into which we're currently assembling
	int       errors;
};

static assembler_t
assembler_alloc(void) {
	return (assembler_t)malloc(sizeof(struct assembler_desc));
}

static assembler_t
assembler_init(assembler_t a) {
	if(a) {
		memset(a, 0, sizeof(struct assembler_desc));
	}
	return a;
}

assembler_t
assembler_new(void) {
	return assembler_init(assembler_alloc());
}

void
assembler_free(assembler_t *pa) {
	if(pa && *pa) {
		free(*pa);
		*pa = NULL;
	}
}

// Sets the section into which the assembler is generating code.
void
assembler_set_section(assembler_t a, section_t s) {
	a->current_section = s;
}

// Attempts to assemble a single source line.
void
assembler_assemble_statement(assembler_t a, section_t inp, statement_t s) {
	slice_t operand_slice = statement_borrow_operand(s);
	struct dc_context_desc context;
	int i, ch;
	struct reader_desc arg_reader;

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
	//
	// Right now, DC X does not support any subtypes.  I plan on supporting R
	// in the future, but for now, keeping things very simple.
	if(context.subtype != ' ') {
		a->errors |= ERRF_BAD_OPERAND;
		return;
	}

	for(i = 0; i < context.duplication; i++) {
		reader_init(&arg_reader, &context.value, inp);

		ch = reader_peek_char(&arg_reader);
		while(isxdigit(ch)) {
			section_append_byte(a->current_section, reader_read_byte_hex(&arg_reader));
			ch = reader_peek_char(&arg_reader);
		}
	}
}

// Answers with the current set of errors.
int
assembler_errors(assembler_t a) {
	return a->errors;
}
