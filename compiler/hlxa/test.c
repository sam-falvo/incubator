#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>

#include "section.h"
#include "hlxa.h"

int
main(int argc, char *argv[]) {
	printf("\"Procedure\",\"Description\",\"Result\"\n");
	printf("assemble,DC parameter X'01',");

	section_t expected;
	section_t actual;
	hlxa_t hlxa;
	bool equal;

	expected = section_new();
	section_append_byte(expected, 0x01);

	actual = section_new();
	hlxa = hlxa_new();
	hlxa_set_section(hlxa, actual);

	hlxa_assemble_line(hlxa, "X'01'");

  equal = section_compare_eq(expected, actual);
	hlxa_free(&hlxa);
	section_free(&actual);
	section_free(&expected);

	if(equal)  printf("PASS\n");
	else       printf("FAIL\n");
}
