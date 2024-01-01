#include <stdint.h>
#include <ctype.h>

#include "reader.h"
#include "dc_context.h"

dc_context_t
dc_context_init(dc_context_t ctx) {
	if(ctx) {
		ctx->duplication = 1;
		ctx->type_ = ' ';
		ctx->subtype = ' ';
		ctx->length = -1;
		ctx->quote = ' ';
		slice_init(&ctx->value);
		ctx->errors = 0;
	}
	return ctx;
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

void
dc_context_validate(dc_context_t ctx, section_t inp) {
	size_t value_length = slice_length(&ctx->value);

	if(ctx->duplication < 1)
		ctx->errors |= DCCTX_ERRF_DUPLICATION;

	switch(ctx->type_) {
	case 'X':
		switch(ctx->subtype) {
		case ' ':
		case 'R':
			break;

		default:
			ctx->errors |= DCCTX_ERRF_SUBTYPE;
			break;
		}

		if(value_length & 1) ctx->errors |= DCCTX_ERRF_VALUE;
		if(!slice_forall_bytes(&ctx->value, inp, is_hexdigit)) ctx->errors |= DCCTX_ERRF_VALUE;
		break;

	default:
		ctx->errors |= DCCTX_ERRF_TYPE;
		break;
	}

	if(ctx->length >= 0) {
		if(ctx->length == 0) ctx->errors |= DCCTX_ERRF_LENGTH;

		switch(ctx->type_) {
		case 'X':
			// Length of field must be at least as large as the number of bytes given (or larger)
			if(ctx->length < (value_length >> 1)) ctx->errors |= DCCTX_ERRF_LENGTH;
			break;
		}
	}

	// This will most likely be checked in the decoder, since it has parsing knowledge.
	switch(ctx->quote) {
	case '"':
  case '\'':
	case '`':
		break;

	default:
		ctx->errors |= DCCTX_ERRF_QUOTE;
		break;
	}
}

int
dc_context_errors(dc_context_t ctx) {
	return ctx->errors;
}

void
dc_context_decode(slice_t s, section_t inp, dc_context_t ctx) {
	struct reader_desc reader;
	int ch, closing_quote;

	dc_context_init(ctx);
	reader_init(&reader, s, inp);

	// Process duplication if present

	ch = reader_peek_char(&reader);
	if(isdigit(ch)) {
		ctx->duplication = reader_read_integer(&reader);
	}

	// Process type

	ctx->type_ = reader_peek_char(&reader);
	reader_next_char(&reader);

	// IF the next character alphabetic and is not P and not L, then it is a subtype.

	ch = reader_peek_char(&reader);
	if(isalpha(ch) && (ch != 'P') && (ch != 'L')) {
		ctx->subtype = ch;
		
		reader_next_char(&reader);
		ch = reader_peek_char(&reader);
	}

	// If the P "program modifier" option is provided, we yield an error.
	if(ch == 'P') {
		ctx->errors |= DCCTX_ERRF_PGMMOD;

		reader_next_char(&reader);
		ch = reader_peek_char(&reader);
	}

	// If the L length modifier is provided, parse it out.
	if(ch == 'L') {
		reader_next_char(&reader);
		ctx->length = reader_read_integer(&reader);
		ch = reader_peek_char(&reader);
	}

	// The next character ought to be the quote for the nominal value field.
	switch(ch) {
	case '\'':
	case '"':
	case '`':
		ctx->quote = ch;
		reader_subslice_string(&reader, &ctx->value);
		closing_quote = reader_peek_char(&reader);
		if(closing_quote != ch) ctx->errors |= DCCTX_ERRF_CLOSE_QUOTE;
		reader_next_char(&reader);
		break;

	default:
		ctx->errors |= DCCTX_ERRF_QUOTE;
		break;
	}
}

