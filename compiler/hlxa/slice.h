#pragma once

#include <stddef.h>
#include <stdbool.h>

#include "section.h"


typedef struct slice_desc *slice_t;
typedef bool slice_pred_fn(uint8_t);

struct slice_desc {
  int start;
  int end;
};


slice_t slice_init(slice_t);
slice_t slice_init_with_bounds(slice_t, int, int);
bool    slice_range_eq(slice_t, int, int);
bool    slice_range_ne(slice_t, int, int);
size_t  slice_length(slice_t);
bool    slice_string_eq(slice_t, section_t, char *);
bool    slice_string_ne(slice_t, section_t, char *);
bool    slice_forall_bytes(slice_t, section_t, slice_pred_fn);

