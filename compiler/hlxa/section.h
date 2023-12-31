#pragma once

typedef struct section_desc *section_t;

section_t section_new(void);
void      section_free(section_t *);
void      section_append_byte(section_t, uint8_t);
bool      section_compare_eq(section_t, section_t);

