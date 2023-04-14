# 3. Bitwise Operators vs Set Types

Date: 2023-04-12

## Status

Accepted

## Context

While developing the compiler, I have run into the problem of deciding whether
I want to represent integers as bit-sets for the purpose of performing general
boolean operations on them (AND, OR, XOR), or if I should just keep operands
represented as normal integers.

Using bit-sets is what Oberon did, and overloaded the arithmetic operators to
operate upon them.

| Expression | Operation                  | Boolean Operation |
|:----------:|:---------------------------|:-----------------:|
| a + b      | Set union                  | a OR b            |
| a - b      | Set difference             | a AND NOT b       |
| a * b      | Set intersection           | a AND b           |
| a / b      | Set difference, symmetric  | a XOR b           |
|   - b      | Set complement             | NOT b             |

The advantage of this is that it reduces the number of operators the compiler
needs to support; as well, it reduces the height of the precedence stack, which
in turn makes implementing the parser simpler.  On the other hand, if we wanted
to do general bit-level manipulation, such as what you'd find when working with
hardware registers, it can be more verbose in the source listing.  Consider a
register with two 4-bit subfields, each of which are comprised of two
additional subfields, a single-bit flag and a 3-bit counter field.

      7   6   5   4   3   2   1   0
    +---+-----------+---+-----------+
    | A | counter A | B | counter B |
    +---+-----------+---+-----------+

Again, using some flavor of pseudo-code, let's create a contrived example of a
module whose sole purpose it is to manipulate these flags and counters.
Observe how manipulating individual bits is convenient enough, but adjusting
the counters turns into a very noisy block of code.

    module RegisterMunger (begin
        global registerValue at SOME_IO_ADDRESS;

        -- public interface

        prc setA* is registerValue := setAFlag(registerValue);
        prc setB* is registerValue := setBFlag(registerValue);
        prc clrA* is registerValue := clrAFlag(registerValue);
        prc clrB* is registerValue := clrBFlag(registerValue);
        prc incA* is registerValue := incACounter(registerValue);
        prc incB* is registerValue := incBCounter(registerValue);

        -- implementation details

        fn result := setAFlag(oldVal) is result := integer(bitset(oldVal) + {7});
        fn result := clrAFlag(oldVal) is result := integer(bitset(oldVal) - {7});
        fn result := setBFlag(oldVal) is result := integer(bitset(oldVal) + {3});
        fn result := clrBFlag(oldVal) is result := integer(bitset(oldVal) - {3});
        fn result := incACounter(oldVal) is (begin
            let fixedBits = integer(bitset(oldVal) * {7, 3..0});
            let oldCounter = integer(bitset(oldVal) * {6..4});
            let newCounter = oldCounter + 0x10;
            result := integer(bitset(newCounter) * {6..4}) + fixedBits
        )
        fn result := incBCounter(oldVal) is (begin
            let fixedBits = integer(bitset(oldVal) * {7..3});
            let oldCounter = integer(bitset(oldVal) * {2..0});
            let newCounter = oldCounter + 0x1;
            result := integer(bitset(newCounter) * {2..0}) + fixedBits
        )
    )

Indeed, if we wanted to refactor the incrementors, could do this to make the
code somewhat clearer:

        fn result := incACounter(oldVal) is result := incAnyCounter(oldVal, {7, 3..0}, {6..4}, 0x10);
        fn result := incBCounter(oldVal) is result := incAnyCounter(oldVal, {7..3}, {2..0}, 0x01);

        fn result := incAnyCounter(oldVal; (bitset)fixedMask, counterMask; preshiftedDelta) is (begin
            let fixedBits = integer(bitset(oldVal) * fixedMask);
            let oldCounter = integer(bitset(oldVal) * counterMask);
            let newCounter = oldCounter + preshiftedDelta;
            result := integer(bitset(newCounter) * counterMask) + fixedBits
        )

But, this is just kicking the can down the road.  As a minor performance
optimization, one might relegate the creation of temporary variables to the
compiler in the hopes that it can do a better job of allocating resources.

        fn result := incAnyCounter(oldVal; (bitset)fixedMask, counterMask; preshiftedDelta) is
            result := 
                integer(bitset(integer(bitset(oldVal) * counterMask) + preshiftedDelta) * counterMask) +
                integer(bitset(oldVal) * fixedMask)

Type-casting doesn't produce any code (in this case), so we can clean up some
of this verbosity by removing unnecessary switching between integer and bitset
arithmetic.  In the process, we can remove an unnecessary masking as well.

        fn result := incAnyCounter(oldVal; (bitset)fixedMask, counterMask; preshiftedDelta) is
            result := integer(
                bitset(oldVal + preshiftedDelta) * counterMask +
                bitset(oldVal) * fixedMask
            )

This is a *lot* cleaner; however, no matter how you slice and dice it, it's
more verbose than the corresponding C code:

        int incAnyCounter(int oldVal, int fixedMask, int counterMask, int preshiftedDelta) {
            return ((oldVal + preshiftedDelta) & counterMask) | (oldVal & fixedMask);
        }

This is especially true if we adopt the Oberon-style programming convention of
relegating all type casting to the SYSTEM module on the assumption that all
casts are considered unsafe:

        import S := SYSTEM;
        ...
        fn result := incAnyCounter(oldVal; (bitset)fixedMask, counterMask; preshiftedDelta) is
            result := S.VAL(integer,
                S.VAL(bitset, oldVal + preshiftedDelta) * counterMask +
                S.VAL(bitset, oldVal) * fixedMask
            )

Thus, the advantages and disadvantages of each method are as follows.

### Advantages of C-style syntax

* Defer implementation of the type system until later, continuing to focus on the parser and code generator.
* We can have more succinct source listings.
* Arguably, a closer mapping to the underlying hardware.

### Disadvantages of C-style syntax

* Less convenient expression of set membership (`x & (1 << N) != 0` vs `N in x`).
* Logical operations apply to all numeric types, increasing chances for mistakes between bitwise and logical operators (e.g., `& and &&`).

### Advantages of Oberon-style types

* A stronger type system sooner.
* Vastly more convenient syntax for checking if element is in a set (`in` operator).
* Type system can be extended in the future (if needed) along several different axes:
  - Support for multi-word bitsets allows for general sets of (contiguous) enumerations
  - Explicitly specifying the universe of a set can help type system verify correct code

### Disadvantages of Oberon-style Types

* Need to implement set literals in lexer, and distinguish them from integers in Tokens.
* Need to support set types in code generator, making `cg_add`, et. al. more complex.
* Need to augment symbol table to record basic type information.
* Need to retrofit parser to perform type checking.
* Need to implement built-in procedures to perform type casting.
  - This implies the need to define and parse procedure calls in general.

## Decision

Despite the C-style notation being easier to implement and promising quicker
return on investments, I still propose making the investment for the
Oberon-inspired type solution.  It introduces more complexity into the
compiler; however, this complexity (a) will be needed eventually *anyway*, and
(b) can actually introduce opportunities for re-use later on, making
longer-term maintenance of the compiler simpler in the long run.

If you study the syntax for Smalltalk-80, you'll see that they express just
about the *entirety* of what the language can do in terms of message sends.
Message sends are isomorphic to subroutine calls (for our purposes); therefore,
if we can express ROIL's primitives in terms of subroutine "calls" to
well-known procedures or functions, then we get the following benefits:

1. The code generator becomes easier to extend in the future, since we need not
   define a new `cg_X` entrypoint for feature X.  (In practice, you'll probably
   dispatch to a subroutine that handles X anyway; but this now becomes an
   *implementation detail*, hidden from the external interface of the code
   generator.)
2. The code generator becomes easier to implement due to a reduction in the
   combinatorial explosion of cases to consider.  For example, consider the +
   operator for integers.  If you want to evaluate loclaVar + rhs, there's a
   separate case for addition, subtraction, multiplication, etc. on the rhs.  All
   we really care about is that the types are correct; we don't care about the
   specific operation.  We want the ability to invoke `cg_item(rhs)` and just
   confirm that the result has the correct type.  We ought not care about the
   specific kind of rhs.
3. Well-known symbol names that conform to normal naming conventions allows for
   the possibility of supporting overloading directly within the language later
   on.  Consider how Ada supports overloading based on parameter types.  Consider
   also that the 65816 lacks a multiply or divide instruction, while an 80286
   supports them.  Thus, the runtimes for these two CPUs would need to differ
   based on what instructions are available to use or not.  Features that are
   missing must be made up for using well-known procedures.
4. Equivalences can be effortlessly supported in the code generator.
   `SET_DIFF(x,y)` can be handled by constructing a temporary `AND(x,NOT(y))` tree
   and recursing into the code generator.  This is equivalent to term-rewriting
   optimization rules found in many other compilers for better known languages.

In some sense, that's what `cg_item` is now; however, primitive operations are
broken out into their own Item enumeration variants.  If we replace these with
a generic Item variant `Call(proc_name:String, arglist:Vec<Item>)`, or
something semantically equivalent to this (like S-expressions), then we can
make the code generator correspondingly more dynamic and easier to extend.

The parser would of course need to be adjusted to work with the new generator
interface.  Before it can produce a `SET_DIFF(x,y)` item, for example, it must
ensure that `x` and `y` are type-compatible, thus relieving the code generator
of the responsibility for testing type compatibility.  Perhaps more
importantly, it must set the resulting type of the "call" to `SET_DIFF`, so
that subsequent parser routines (representing expressions that *called*
`SET_DIFF` in the first place) can also quickly determine type compatibility of
its sub-expressions.  In other words, the parser, which currently works with an
un-annotated parse tree, must be changed to use an *annotated* parse tree.

I propose the following steps be taken to move forward with an Oberon-style,
type-based solution.  I follow the basic philosophy of "working backwards."

1.  Update code generator to accept programs in the form of call trees instead of Item trees.
    1.  **DONE**.  Based on the current set of acceptance tests, define a set of unit tests for the current code generator.  Refactor production code accordingly.
    1.  Introduce a new set of tests, essentially identical to the current set, but which expresses the same syntax tree using a generic Apply Item variant.  Alter production code so that handling these new call trees produces identical results to the older set of tests.  Leave legacy code in place, so that acceptance tests remain valid.
    1.  At this point, the code generator should be support the Apply Item variant for expression evaluation.  Bitsets are not yet supported.  Acceptance tests still pass.
1.  Update the parser to use the Apply Item instead of dedicated Item nodes.  At each step, acceptance tests must continue to pass.
    1.  Based on the current set of acceptance tests, define a set of unit tests for the current parser.  Refactor production code accordingly.
    1.  Incrementally alter the set of unit tests to check for use of the Apply Item variant.  Refactor production code accordingly.
    1.  At this point, the parser has been retrofitted and relies solely on the Apply variant for expression evaluation.
1.  Retire obsolete code in Code Generator.  Acceptance tests must continue to pass.
    1.  Incrementally remove the production code supporting the older set of unit tests.  Confirm relevant tests fail as expected.
    1.  Remove tests once confirmed.
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

Compiler becomes significantly more complex.  However, core facilities
necessary to support more complicated types will have been laid.  There is a
literal *ton* to accomplish with this effort, but the payoff should be (I hope)
worth it.

