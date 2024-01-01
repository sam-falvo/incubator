#pragma once

#include <stdint.h>

#include "slice.h"

typedef struct dc_context_desc *dc_context_t;

struct dc_context_desc {
	uint16_t          duplication;
	char              type_;
	char              subtype;
	int               length;
	char              quote;
	struct slice_desc value;
	int               errors;
};

#define DCCTX_ERRF_DUPLICATION   0x0001
#define DCCTX_ERRF_TYPE          0x0002
#define DCCTX_ERRF_SUBTYPE       0x0004
#define DCCTX_ERRF_LENGTH        0x0008
#define DCCTX_ERRF_QUOTE         0x0010
#define DCCTX_ERRF_VALUE         0x0020
#define DCCTX_ERRF_CLOSE_QUOTE   0x0040
#define DCCTX_ERRF_PGMMOD        0x0080

dc_context_t dc_context_init(dc_context_t ctx);
void         dc_context_validate(dc_context_t ctx, section_t inp);
int          dc_context_errors(dc_context_t ctx);
void         dc_context_decode(slice_t s, section_t inp, dc_context_t ctx);
