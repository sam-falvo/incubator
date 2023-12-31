# 2. Handling DC constant syntax

Date: 2023-12-31

## Status

Tentative

## Context

IBM HLASM syntax for constants can be pretty complex.  I reproduce it below for convenience.  It was acquired originally from [IBM's HLASM documentation](https://www.ibm.com/docs/en/zos/2.1.0?topic=statements-dc-instruction#dc).

    DC Statement

    >>----+---------------+----- DC --+--> operand --+-----><
          |               |           |              |
          `---> symbol ---'           `----- , ------'


    Operand

    >>----+--------------------+---> type ---+------------------+---+----------------+---+------------------+---> value ---><
          |                    |             |                  |   |                |   |                  |
          `---> duplication ---'             `---> extension ---'   `---> pgm_mod ---'   `---> modifiers ---'

IBM's given example:

    FLTBUF    DC 10EBP(7)L2'12'

indicates a buffer that:
- is 10 elements long (10)
- contains binary (B) floating point (E) values
- Each floating point value is 16-bits wide (L2)
$$$$$$$$$$$$
Thus, the buffer emitted is 20 bytes long, containing 10 16-bit floating point values, each of which holds the value 12.
FLTBUF has a program modifier of \$00000007.  This would probably be of value to a linker script or debugger somehow.

| Field       | Interpretation (if present)                                                                                   | HLXA Support |
|:------------|:--------------------------------------------------------------------------------------------------------------|:------------:|
| duplication | Causes the nominal_value to be generated the number of times indicated by this factor.                        | Y            |
| type        | Further determines the type of constant the nominal_value represents.                                         | Y            |
| extension   | Determines some of the characteristics of the constant.                                                       | Y            |
| pgm_mod     | assign a programmer determined 32 bit value to the symbol naming the DC instruction, if a symbol was present. | N            |
| modifiers   | Describes the length, the scaling, and the exponent of the nominal_value.                                     | Y            |
| value       | Defines the value of the constant.                                                                            | Y            |

I'm not sure what pgm_mod is used for in practice; I've been unable to locate a demonstration of its use online.
Even so, just in case it becomes useful in the future, type P and extension P are both reserved for future use.
For now, though, its use will create syntax errors.

**NOTE:** IBM defines three modifiers (length, scale, exponent).  HLXA only supports length, since floating point values are not yet supported.

The question is, how do we validate an operand before generating code?

## Decision

I think the right approach is to first decode the operand string into a data structure, whose member values may well be nonsensical.
Then, we validate the structure's fields to make sure they make sense.
E.g., for a C-type constant, the number of characters in the string can vary arbitrarily; however,
for X-type constants, the number of characters in the string must be even, and they must all be hexadecimal digits.
Once validated, we then pass this structure on to a third function, whose job is to interpret this structure and generate the intended result.

I suspect that we will define a structure which contains at least the following fields:

| Field       | Type | Default | Purpose                                                                |
|:------------|:-----|:-------:|:-----------------------------------------------------------------------|
| duplication | int  | 1       | Decodes the duplication parameter, if given.                           |
| type        | char | none    | Records the basic type of the constant.  Required.                     |
| extension   | char | space   | Set to the extension character, if present.                            |
| length      | int  | -1      | Set to the length modifier if provided.                                |
| quote       | char | none    | Selects the kind of quote character used to enclose the nominal value. |
| value_start | int  | none    | Index in input buffer of first character of nominal value.             |
| value_stop  | int  | none    | Index in input buffer of what we hope is the terminating quote.        |
| errors      | set? | empty   | Set of errors discovered during validation                             |

In the plan above, errors are recorded in a bitmap, so figure at most 16 kinds of errors can be found.

| Error | Description                                                             |
|:-----:|:------------------------------------------------------------------------|
| 0     | Duplication factor too large.                                           |
| 1     | Invalid type.                                                           |
| 2     | Invalid extension for given type.                                       |
| 3     | Length doesn't make sense for given type/extension.                     |
| 4     | Missing quote character.                                                |
| 5     | Nominal value doesn't make sense (e.g., non-hex digit in hex constant). |
| 6     | Terminating quote doesn't match opening quote.                          |
| 7     | Program modifier provided but is not supported.                         |

To facilitate better error reporting, we might want to also record the indices of where the different fields start and/or end as well (e.g., duplication_start/duplication_end, etc.).
At this point, bounding indices for different fields are probably best described in a "slice" data type.
But for the purposes of raw validation, the fields above seem to be the minimum required.

## Consequences

We need to provide input strings in section_t buffers.  This provides a consistent mechanism for expressing an array of characters for slices.

We need to provide the concept of a slice, which points back to a section_t and provides a start and stop range index.

Given a string, we need a function that decodes it into an optional label, mnemonic, and operand fields/slices.  Extra credit if we can also record a comment field as well.
Similar to the decision above, any errors encountered would be recorded as a bit-set.

Once we have an operand field identified, we can decode and validate it.  Assuming it passes validation, we can then emit the required code accordingly.

This implies that once a handler for a mnemonic is done processing operand bytes, the remainder of its field is ignored (hence, no particular designator is required for most comments).

