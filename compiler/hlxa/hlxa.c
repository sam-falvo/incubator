#include <sys/stat.h>

#include <stdio.h>
#include <string.h>
#include <stdint.h>
#include <unistd.h>
#include <fcntl.h>
#include <stdbool.h>

#include "assembler.h"


typedef void assembly_good_t(char *, assembler_t);


#define OBJ_FILE_PERMS   0660

#define MAX_INP_FILENAME_SIZE   256
#define MAX_OBJ_FILENAME_SIZE   (MAX_INP_FILENAME_SIZE+3)


static bool
file_exists(char *filename) {
	struct stat sb;
	int erc;

	erc = stat(filename, &sb);
	return erc >= 0;
}


static void
derive_output_filename_from_input(char *inp_filename, char *out_filename, size_t out_length) {
	memset(out_filename, 0, out_length);
	strncpy(out_filename, inp_filename, out_length - 3);
	strcat(out_filename, ".o");
}


static void
emit_object_file(char *out_filename, assembler_t a) {
	int fd;
	size_t actual, expected;
	uint16_t file_type = 500;
  uint16_t slen;
	section_t s;

	fd = open(out_filename, O_CREAT | O_RDWR, OBJ_FILE_PERMS);
	if(fd < 0) {
		fprintf(stderr, "** Unable to open or create file: %s\n", out_filename);
		return;
	}

	expected = 2;
	actual = write(fd, (void *)&file_type, 2);
	if(actual != 2) goto bail;

	s = assembler_get_section(a);
	slen = section_length(s);
	actual = write(fd, (void *)&slen, 2);
	if(actual != 2) goto bail;

	expected = slen;
	actual = write(fd, (void *)section_borrow_buffer(s), slen);

bail:
	if(actual != expected) fprintf(stderr, "** write() error\n");
	close(fd);
}


static void
assemble_input_file(char *inp_filename, assembly_good_t good_fn) {
	assembler_t assembler;
	section_t default_section, input_section;
	statement_t statement;
	FILE *fp = NULL;
	int errs, error_count = 0;
	int line = 1;

	statement = statement_new();
	if(!statement) goto bail;

	default_section = section_new();
	if(!default_section) goto bail;

	input_section = section_new();
	if(!input_section) goto bail;

	assembler = assembler_new();
	if(!assembler) goto bail;

	assembler_set_section(assembler, default_section);

	fp = fopen(inp_filename, "r");
	if(!fp) goto bail;

	while(section_refill_from_file(input_section, fp)) {
		section_debug_print_buffer(stderr, input_section);
		statement_decode(input_section, statement);
		// TODO: statement_errors() will always return 0 as of this writing.
		assembler_assemble_statement(assembler, input_section, statement);

		errs = assembler_errors(assembler);
		if(errs & ERRF_MISSING_OPERAND) {
			++ error_count;
			fprintf(stderr, "%s:%d:Missing operand\n", inp_filename, line);
		}
		if(errs & ERRF_UNKNOWN_MNEMONIC) {
			++ error_count;
			fprintf(stderr, "%s:%d:Unknown mnemonic\n", inp_filename, line);
		}
		if(errs & ERRF_BAD_OPERAND) {
			++ error_count;
			fprintf(stderr, "%s:%d:Bad or malformed operand\n", inp_filename, line);
		}

		++ line;
	}

	if(!error_count) good_fn(inp_filename, assembler);

bail:
	if(fp) fclose(fp);

	assembler_free(&assembler);
	section_free(&input_section);
	section_free(&default_section);
	statement_free(&statement);
}


void
on_successful_assembly(char *inp_filename, assembler_t a) {
	char out_filename[MAX_OBJ_FILENAME_SIZE];

	derive_output_filename_from_input(inp_filename, out_filename, MAX_OBJ_FILENAME_SIZE);
	if(file_exists(out_filename)) fprintf(stderr, "** Warning: overwriting %s\n", out_filename);
	emit_object_file(out_filename, a);
}


int
main(int argc, char *argv[])
{
	for(int i = 1; i < argc; ++i) assemble_input_file(argv[i], &on_successful_assembly);
}
