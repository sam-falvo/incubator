#pragma once

typedef struct assembler_desc *assembler_t;

assembler_t hlxa_new(void);
void        hlxa_free(assembler_t *);
void        hlxa_set_section(assembler_t, section_t);
void        hlxa_assemble_line(assembler_t, char *);

