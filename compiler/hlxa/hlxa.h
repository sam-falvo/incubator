#pragma once

typedef struct hlxa_desc *hlxa_t;

hlxa_t hlxa_new(void);
void   hlxa_free(hlxa_t *);
void   hlxa_set_section(hlxa_t, section_t);
void   hlxa_assemble_line(hlxa_t, char *);

