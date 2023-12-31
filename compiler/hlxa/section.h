#pragma once

#include <stddef.h>
#include <stdint.h>

typedef struct section_desc *section_t;

section_t section_new(void);
section_t section_new_from_string(char *);
void      section_free(section_t *);
void      section_append_byte(section_t, uint8_t);
bool      section_compare_eq(section_t, section_t);
void      section_append_string(section_t, char *);
int       section_byte_at(section_t, int);
size_t    section_length(section_t);

