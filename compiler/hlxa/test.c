#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>

#include "slice.h"
#include "section.h"
#include "statement.h"
#include "hlxa.h"

void
print_table_header(void) {
	printf("\"Procedure\",\"Description\",\"Result\"\n");
}


void
test_hlxa_assemble_statement_01(void) {
	bool equal = false;
	hlxa_t hlxa;
	section_t actual;
	section_t expected;
	section_t input_section;
	statement_t statement;

	printf("hlxa_assemble_statement,DC X'01',");

	statement = statement_new();
	input_section = section_new_from_string("  DC X'01'");
	statement_decode(input_section, statement);
	if(statement_errors(statement)) goto bail;

	expected = section_new();
	if(!expected) goto bail;
	section_append_byte(expected, 0x01);

	actual = section_new();
	if(!actual) goto bail;

	hlxa = hlxa_new();
	if(!hlxa) goto bail;

	hlxa_set_section(hlxa, actual);
	hlxa_assemble_statement(hlxa, input_section, statement);

	if(hlxa_errors(hlxa)) goto bail;

  equal = section_compare_eq(expected, actual);

bail:
	hlxa_free(&hlxa);
	section_free(&actual);
	section_free(&expected);
	section_free(&input_section);
	statement_free(&statement);

	if(equal)  printf("PASS\n");
	else       printf("FAIL\n");
}


void
test_hlxa_assemble_statement_02(void) {
	bool equal = false;
	hlxa_t hlxa;
	section_t actual;
	section_t expected;
	section_t input_section;
	statement_t statement;

	printf("hlxa_assemble_line,DC X'0123',");

	statement = statement_new();
	input_section = section_new_from_string("  DC X'0123'");
	statement_decode(input_section, statement);
	if(statement_errors(statement)) goto bail;

	expected = section_new();
	if(!expected) goto bail;
	section_append_byte(expected, 0x01);
	section_append_byte(expected, 0x23);

	actual = section_new();
	if(!actual) goto bail;

	hlxa = hlxa_new();
	if(!hlxa) goto bail;

	hlxa_set_section(hlxa, actual);
	hlxa_assemble_statement(hlxa, input_section, statement);

	if(hlxa_errors(hlxa)) goto bail;

  equal = section_compare_eq(expected, actual);

bail:
	hlxa_free(&hlxa);
	section_free(&actual);
	section_free(&expected);
	section_free(&input_section);
	statement_free(&statement);

	if(equal)  printf("PASS\n");
	else       printf("FAIL\n");
}


void
test_hlxa_assemble_statement_03(void) {
	bool equal = false;
	hlxa_t hlxa;
	section_t actual;
	section_t expected;
	section_t input_section;
	statement_t statement;

	printf("hlxa_assemble_line,DC X'01234567',");

	statement = statement_new();
	input_section = section_new_from_string("  DC X'01234567'");
	statement_decode(input_section, statement);
	if(statement_errors(statement)) goto bail;

	expected = section_new();
	if(!expected) goto bail;
	section_append_byte(expected, 0x01);
	section_append_byte(expected, 0x23);
	section_append_byte(expected, 0x45);
	section_append_byte(expected, 0x67);

	actual = section_new();
	if(!actual) goto bail;

	hlxa = hlxa_new();
	if(!hlxa) goto bail;

	hlxa_set_section(hlxa, actual);
	hlxa_assemble_statement(hlxa, input_section, statement);

  equal = section_compare_eq(expected, actual);

bail:
	hlxa_free(&hlxa);
	section_free(&actual);
	section_free(&expected);
	section_free(&input_section);
	statement_free(&statement);

	if(equal)  printf("PASS\n");
	else       printf("FAIL\n");
}


void
test_hlxa_assemble_statement_04(void) {
	bool equal = false;
	hlxa_t hlxa;
	section_t actual;
	section_t expected;
	section_t input_section;
	statement_t statement;

	printf("hlxa_assemble_line,DC X'0123456',");

	statement = statement_new();
	input_section = section_new_from_string("  DC X'0123456'");
	statement_decode(input_section, statement);
	if(statement_errors(statement)) goto bail;

	expected = section_new();
	if(!expected) goto bail;

	actual = section_new();
	if(!actual) goto bail;

	hlxa = hlxa_new();
	if(!hlxa) goto bail;

	hlxa_set_section(hlxa, actual);
	hlxa_assemble_statement(hlxa, input_section, statement);
	if(!hlxa_errors(hlxa)) goto bail;

	// The above is a syntax error; so we expect an
	// empty section due to an error.
  equal = section_compare_eq(expected, actual);

bail:
	hlxa_free(&hlxa);
	section_free(&actual);
	section_free(&expected);
	section_free(&input_section);
	statement_free(&statement);

	if(equal)  printf("PASS\n");
	else       printf("FAIL\n");
}


void
test_hlxa_assemble_statement_05(void) {
	bool equal = false;
	hlxa_t hlxa;
	section_t actual;
	section_t expected;
	section_t input_section;
	statement_t statement;

	printf("hlxa_assemble_line,DC '01234567',");

	statement = statement_new();
	input_section = section_new_from_string("  DC '01234567'");
	statement_decode(input_section, statement);
  if(statement_errors(statement)) goto bail;

	expected = section_new();
	if(!expected) goto bail;

	actual = section_new();
	if(!actual) goto bail;

	hlxa = hlxa_new();
	if(!hlxa) goto bail;

	hlxa_set_section(hlxa, actual);
	hlxa_assemble_statement(hlxa, input_section, statement);
	if(!hlxa_errors(hlxa)) goto bail;

	// The above is a syntax error; so we expect an
	// empty section due to an error.
  equal = section_compare_eq(expected, actual);

bail:
	hlxa_free(&hlxa);
	section_free(&actual);
	section_free(&expected);
	section_free(&input_section);
	statement_free(&statement);

	if(equal)  printf("PASS\n");
	else       printf("FAIL\n");
}


void
test_hlxa_assemble_statement_06(void) {
	bool equal = false;
	hlxa_t hlxa;
	section_t actual;
	section_t expected;
	section_t input_section;
	statement_t statement;

	printf("hlxa_assemble_line,DC X'01234567\\\",");

	statement = statement_new();
	input_section = section_new_from_string("  DC X'01234567\"");
	statement_decode(input_section, statement);
	if(statement_errors(statement)) goto bail;

	expected = section_new();
	if(!expected) goto bail;

	actual = section_new();
	if(!actual) goto bail;

	hlxa = hlxa_new();
	if(!hlxa) goto bail;

	hlxa_set_section(hlxa, actual);
	hlxa_assemble_statement(hlxa, input_section, statement);
	if(!hlxa_errors(hlxa)) goto bail;

	// The above is a syntax error; so we expect an
	// empty section due to an error.
  equal = section_compare_eq(expected, actual);

bail:
	hlxa_free(&hlxa);
	section_free(&actual);
	section_free(&expected);
	section_free(&input_section);
	statement_free(&statement);

	if(equal)  printf("PASS\n");
	else       printf("FAIL\n");
}


void
test_statement_decode_01(void) {
	bool passed = false;
	section_t statement_sect;
	statement_t statement;

	printf("statement_decode,XXXyyyzzz  XXXX  01234567,");

	// Given an assembly listing input of:
	//
	//           1    1    2    2
	// 0....5....0....5....0....5....
	//
	// XXXyyyzzz  XXXX  01234567
	//
	// We expect the various field slices to match as follows:
	//
	// label start = 0
	// label stop = 9
	// mnem start = 11
	// mnem stop = 15
	// oper start = 17
	// oper stop = 25

	statement_sect = section_new_from_string("XXXyyyzzz  XXXX  01234567");
  statement = statement_new();
	if(!statement) goto bail;
	statement_decode(statement_sect, statement);

	if(statement_errors(statement)) goto bail;

	if(slice_range_ne(statement_borrow_label(statement), 0, 9)) goto bail;
	if(slice_range_ne(statement_borrow_mnemonic(statement), 11, 15)) goto bail;
	if(slice_range_ne(statement_borrow_operand(statement), 17, 25)) goto bail;

	passed = true;

bail:
	statement_free(&statement);
	section_free(&statement_sect);

	if(passed) printf("PASS\n");
	else       printf("FAIL\n");
}


void
test_statement_decode_02(void) {
	bool passed = false;
	section_t statement_sect;
	statement_t statement;

	printf("statement_decode,           XXXX  01234567,");

	// Given an assembly listing input of:
	//
	//           1    1    2    2
	// 0....5....0....5....0....5....
	//
	//            XXXX  01234567
	//
	// We expect the various field slices to match as follows:
	//
	// label start = -1
	// label stop = -1
	// mnem start = 11
	// mnem stop = 15
	// oper start = 17
	// oper stop = 25

	statement_sect = section_new_from_string("           XXXX  01234567");
  statement = statement_new();
	if(!statement) goto bail;
	statement_decode(statement_sect, statement);

	if(statement_errors(statement)) goto bail;

	if(slice_range_ne(statement_borrow_label(statement), -1, -1)) goto bail;
	if(slice_range_ne(statement_borrow_mnemonic(statement), 11, 15)) goto bail;
	if(slice_range_ne(statement_borrow_operand(statement), 17, 25)) goto bail;

	passed = true;

bail:
	statement_free(&statement);
	section_free(&statement_sect);

	if(passed) printf("PASS\n");
	else       printf("FAIL\n");
}


void
test_statement_decode_03(void) {
	bool passed = false;
	section_t statement_sect;
	statement_t statement;

	printf("statement_decode,XXXyyyzzz  XXXX          ,");

	// Given an assembly listing input of:
	//
	//           1    1    2    2
	// 0....5....0....5....0....5....
	//
	// XXXyyyzzz  XXXX
	//
	// We expect the various field slices to match as follows:
	//
	// label start = 0
	// label stop = 9
	// mnem start = 11
	// mnem stop = 15
	// oper start = -1
	// oper stop = -1

	statement_sect = section_new_from_string("XXXyyyzzz  XXXX          ");
  statement = statement_new();
	if(!statement) goto bail;
	statement_decode(statement_sect, statement);

	if(statement_errors(statement)) goto bail;

	if(slice_range_ne(statement_borrow_label(statement), 0, 9)) goto bail;
	if(slice_range_ne(statement_borrow_mnemonic(statement), 11, 15)) goto bail;
	if(slice_range_ne(statement_borrow_operand(statement), -1, -1)) goto bail;

	passed = true;

bail:
	statement_free(&statement);
	section_free(&statement_sect);

	if(passed) printf("PASS\n");
	else       printf("FAIL\n");
}


void
test_statement_decode_04(void) {
	bool passed = false;
	section_t statement_sect;
	statement_t statement;

	printf("statement_decode,           XXXX          ,");

	// Given an assembly listing input of:
	//
	//           1    1    2    2
	// 0....5....0....5....0....5....
	//
	//            XXXX
	//
	// We expect the various field slices to match as follows:
	//
	// label start = -1
	// label stop = -1
	// mnem start = 11
	// mnem stop = 15
	// oper start = -1
	// oper stop = -1

	statement_sect = section_new_from_string("           XXXX          ");
  statement = statement_new();
	if(!statement) goto bail;
	statement_decode(statement_sect, statement);

	if(statement_errors(statement)) goto bail;

	if(slice_range_ne(statement_borrow_label(statement), -1, -1)) goto bail;
	if(slice_range_ne(statement_borrow_mnemonic(statement), 11, 15)) goto bail;
	if(slice_range_ne(statement_borrow_operand(statement), -1, -1)) goto bail;

	passed = true;

bail:
	statement_free(&statement);
	section_free(&statement_sect);

	if(passed) printf("PASS\n");
	else       printf("FAIL\n");
}


void
test_statement_decode_05(void) {
	bool passed = false;
	section_t statement_sect;
	statement_t statement;

	printf("statement_decode,XXXyyyzzz                ,");

	// Given an assembly listing input of:
	//
	//           1    1    2    2
	// 0....5....0....5....0....5....
	//
	// XXXyyyzzz
	//
	// We expect the various field slices to match as follows:
	//
	// label start = 0
	// label stop = 9
	// mnem start = -1
	// mnem stop = -1
	// oper start = -1
	// oper stop = -1

	statement_sect = section_new_from_string("XXXyyyzzz                ");
  statement = statement_new();
	if(!statement) goto bail;
	statement_decode(statement_sect, statement);

	if(statement_errors(statement)) goto bail;

	if(slice_range_ne(statement_borrow_label(statement), 0, 9)) goto bail;
	if(slice_range_ne(statement_borrow_mnemonic(statement), -1, -1)) goto bail;
	if(slice_range_ne(statement_borrow_operand(statement), -1, -1)) goto bail;

	passed = true;

bail:
	statement_free(&statement);
	section_free(&statement_sect);

	if(passed) printf("PASS\n");
	else       printf("FAIL\n");
}


void
test_statement_decode_06(void) {
	bool passed = false;
	section_t statement_sect;
	statement_t statement;

	printf("statement_decode,                         ,");

	// Given an assembly listing input of:
	//
	//           1    1    2    2
	// 0....5....0....5....0....5....
	//
	// (blank line)
	//
	// We expect the various field slices to match as follows:
	//
	// label start = -1
	// label stop = -1
	// mnem start = -1
	// mnem stop = -1
	// oper start = -1
	// oper stop = -1

	statement_sect = section_new_from_string("                         ");
  statement = statement_new();
	if(!statement) goto bail;
	statement_decode(statement_sect, statement);

	if(statement_errors(statement)) goto bail;

	if(slice_range_ne(statement_borrow_label(statement), -1, -1)) goto bail;
	if(slice_range_ne(statement_borrow_mnemonic(statement), -1, -1)) goto bail;
	if(slice_range_ne(statement_borrow_operand(statement), -1, -1)) goto bail;

	passed = true;

bail:
	statement_free(&statement);
	section_free(&statement_sect);

	if(passed) printf("PASS\n");
	else       printf("FAIL\n");
}


int
main(int argc, char *argv[]) {
	print_table_header();

	test_statement_decode_01();
	test_statement_decode_02();
	test_statement_decode_03();
	test_statement_decode_04();
	test_statement_decode_05();
	test_statement_decode_06();

	test_hlxa_assemble_statement_01();
	test_hlxa_assemble_statement_02();
	test_hlxa_assemble_statement_03();
	test_hlxa_assemble_statement_04();
	test_hlxa_assemble_statement_05();
	test_hlxa_assemble_statement_06();
}
