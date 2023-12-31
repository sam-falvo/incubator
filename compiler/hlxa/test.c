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
	bool equal = false;

	expected = section_new();
	if(!expected) goto bail;
	section_append_byte(expected, 0x01);

	actual = section_new();
	if(!actual) goto bail;

	hlxa = hlxa_new();
	if(!hlxa) goto bail;

	hlxa_set_section(hlxa, actual);
	hlxa_assemble_line(hlxa, "X'01'");

  equal = section_compare_eq(expected, actual);

bail:
	hlxa_free(&hlxa);
	section_free(&actual);
	section_free(&expected);

	if(equal)  printf("PASS\n");
	else       printf("FAIL\n");
}
