# 4. Uniform representation of functions and procedures in Apply nodes

Date: 2023-04-14

## Status

Accepted

## Context

As I write this ADR, expression evaluation is internally represented as a tree
of Item enumeration records.  Each compiler-supported operation has its own
unique Item variant, which is `match`ed inside the top-level code generator,
`cg_item`, to dispatch generation to Item-specific handlers.  Each of these
items might contain one or more sub-Items.  One can think of these sub-items as
operands to an operator, or parameters to a procedure call.  This will
frequently involve recursive calls back into `cg_item`.

This approach works great from the perspective that the Rust compiler enforces
parity between what the parser can produce and the parse tree accepted by the
code generator.  When writing ADR-0003, however, I had it in my mind that
adopting a uniform "apply" operation, with the function specified as a
parameter, would simplify the code generator and parser alike.  My original
intent in writing this ADR was to come up with the data type that would
represent the function specifier.  However, upon critically thinking about it,
I no longer think this step is necessary.  We can retain the existing set of
Item definitions and the interface between the parser and code generator, while
still proceeding to implement type safety as per the remainder of ADR-0003.

## Decision

Taking the time to scaffold the code generator and parser with unit tests still makes a lot of sense to me.  However, replacing every expression evaluator with a generic Item::Apply type, while still doable in the future, just does not seem to make sense to me now.

Therefore, I amend the set of steps to be taken as follows:

1.  Update code generator to accept programs in the form of call trees instead of Item trees.
    1.  **DONE**.  Based on the current set of acceptance tests, define a set of unit tests for the current code generator.  Refactor production code accordingly.
    1.  **SKIP**.  Introduce a new set of tests, essentially identical to the current set, but which expresses the same syntax tree using a generic Apply Item variant.  Alter production code so that handling these new call trees produces identical results to the older set of tests.  Leave legacy code in place, so that acceptance tests remain valid.
1.  Update the parser to use the Apply Item instead of dedicated Item nodes.  At each step, acceptance tests must continue to pass.
    1.  Based on the current set of acceptance tests, define a set of unit tests for the current parser.  Refactor production code accordingly.
    1.  **SKIP**.  Incrementally alter the set of unit tests to check for use of the Apply Item variant.  Refactor production code accordingly.
1.  **SKIP**.  Retire obsolete code in Code Generator.  Acceptance tests must continue to pass.
1.  Introduce set operations in Code Generator.
    1.  All current tests assume integer, so alter all existing tests to initialize Items to integers.  Add type field to Items as needed.
    1.  Introduce a new test for `a+b` on bitsets.  Refactor production code accordingly.
    1.  Introduce a new test for `a-b` on bitsets.  Refactor production code accordingly.
    1.  Introduce a new test for `a*b` on bitsets.  Refactor production code accordingly.
    1.  Introduce a new test for `a/b` on bitsets.  Refactor production code accordingly.
    1.  Introduce a new test for `-b` on a bitset, using bitsets instead of integers.  Refactor production code accordingly.
1.  Introduce '..' token in lexer.
1.  Introduce set literals into the parser.  Items returned, if any, should have type bitset.
    1.  Test for {}.
    1.  Test for {1}.
    1.  Test for {1, 2, 3}.
    1.  Test for {1..3}.
    1.  Test for {1, 2..4}.
    1.  Test for {1..3, 4}.
    1.  Test for {1..3, 5..7}.
    1.  Test that {16} yields a compile-time error.
    1.  Test that {-1} yields a compile-time error.
1.  Introduce type checking in parser.  Compatible types should pass typecheck and return an Apply item; else, a type error.
    1.  Add unit test for `a+b` for (int, int) => Apply(ADD,...), (set, set) => Apple(BITOR,...), and (int, set) => TypeError.
    1.  Add unit test for `a-b` for (int, int) => Apply(SUB,...), (set, set) => Apple(BITBIC,...), and (int, set) => TypeError.
    1.  Add unit test for `a*b` for (int, int) => Apply(MUL,...), (set, set) => Apple(BITAND,...), and (int, set) => TypeError.
    1.  Add unit test for `a/b` for (int, int) => Apply(DIV,...), (set, set) => Apple(BITXOR,...), and (int, set) => TypeError.
    1.  Add unit test for `-b` for int => Apply(NEG,...) and set => Apply(BITNOT, ...).
1.  Support set literal notation in let-bindings.
    1.  Current let-binding handlers assume rhs is integer; create unit tests to confirm this.
    1.  Create unit tests for let-binding with bitset rhs.  Confirm type of Item::DeclareLocal is bitset.  Confirm variable created is also bitset.
1.  Introduce new acceptance tests.
    1.  `let x = {4..11}; let y = {0..7}; x+y` ==> `LDA _y; ORA _x`
    1.  `let x = {4..11}; let y = {0..7}; x-y` ==> `LDA _y; EOR #$FFFF; AND _x`
    1.  `let x = {4..11}; let y = {0..7}; x*y` ==> `LDA _y; AND _x`
    1.  `let x = {4..11}; let y = {0..7}; x/y` ==> `LDA _y; EOR _x`
    1.  `let x = {4..11}; -x` ==> `LDA _x; EOR #$FFFF`
    1.  `let x = {4..11}; x+{0..7}` ==> `LDA #$00FF; ORA _x`
    1.  `let x = {4..11}; x-{0..7}` ==> `LDA #$FF00; AND _x`
    1.  `let x = {4..11}; x*{0..7}` ==> `LDA #$00FF; AND _x`
    1.  `let x = {4..11}; x/{0..7}` ==> `LDA #$00FF; EOR _x`
    1.  `let x = {4..11}; {0..7}+x` ==> `LDA _x; ORA #$00FF`
    1.  `let x = {4..11}; {0..7}-x` ==> `LDA _x; EOR #$FFFF; AND #$00FF`
    1.  `let x = {4..11}; {0..7}*x` ==> `LDA _x; AND #$00FF`
    1.  `let x = {4..11}; {0..7}/x` ==> `LDA _x; EOR #$00FF`
    1.  `-{4..11}` ==> `LDA #$F00F`
    1.  `-{}` ==> `LDA #$FFFF`

## Consequences

Skipping the transition to using Item::Apply allows us to focus more on the delivery of value.  We can always migrate towards Item::Apply if/when it ever becomes necessary in the future; however, I cannot convince myself that this will become necessary.

My original justification for Item::Apply was that it unified the namespace between intrinsic and extrinsic operations.  This could be useful for, e.g., supporting overloading when calling a function or procedure, and it seems like it would simplify specializing a code generator for a target processor architecture.  The practical cost of this is the loss of compiler-enforced parity between code generator and parser, and more difficult and resource-consuming methods of dispatching to specific code generator subroutines.

While I still like the idea, the implementation (at least in Rust) does not buy its own way.  I have to kill this darling.

