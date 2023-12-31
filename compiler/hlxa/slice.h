#pragma once

#include <stddef.h>
#include <stdbool.h>

#include "section.h"


typedef struct slice_desc *slice_t;

struct slice_desc {
  int start;
  int end;
};


slice_t slice_init(slice_t);
bool    slice_range_eq(slice_t, int, int);
bool    slice_range_ne(slice_t, int, int);
size_t  slice_length(slice_t);
bool    slice_string_eq(slice_t, section_t, char *);
bool    slice_string_ne(slice_t, section_t, char *);
