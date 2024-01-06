#pragma once

#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

typedef struct section_desc *section_t;

section_t section_new(void);
section_t section_new_from_string(char *);
void      section_free(section_t *);
void      section_append_byte(section_t, uint8_t);
bool      section_compare_eq(section_t, section_t);
void      section_append_string(section_t, char *);
int       section_byte_at(section_t, int);
size_t    section_length(section_t);
bool      section_memcmp_eq(section_t, int, uint8_t *, size_t);
bool      section_memcmp_ne(section_t, int, uint8_t *, size_t);
bool      section_refill_from_file(section_t, FILE *);
uint8_t * section_borrow_buffer(section_t);

char *    section_byte_address_fixme(section_t, int);
void      section_debug_print_buffer(FILE *, section_t);
