# 2. Locals

Date: 2023-03-12

## Status

Accepted

## Context

Right now, the ROIL compiler only understands positive or negative constants.
There's no support for arithmetic, logic, or other functions.  Indeed, what is
even the point, if there is no place to store the intermediate computations?
To do that, we need local variables.

Alternative 1.  Using Direct-Page to isolate data from control stack.

We could use the 65816's programmable direct-page base register, D, as a frame
pointer just for data.  Unfortunately, direct page references are strictly
unsigned; therefore, to cleanly delineate argument inputs from local variables
and subroutine parameters, we must apply a fixed bias to the value in D.  As
long as we just affect D using relative displacements, though, it ought not be
a concern.

For example, suppose we have a function to compute the n-th Fibonacci sequence
value (using Rust-y pseudo-code) along with a function to sum eight numbers:
```
fn fibonacci(n: mut u16) -> u16 {
    let mut a: u16 = 0;
    let mut b: u16 = 1;

    while n > 0 {
        a, b = b, a+b;
        n = n - 1;
    }

    b
}

fn sum8(a: u16, b: u16, c: u16, d: u16,
        e: u16, f: u16, g: u16, h: u16) -> u16 {
    a + b + c + d + e + f + g + h
}

fn nested() -> u16 {
    sum8(
        fibonacci(0),
        fibonacci(1),
        fibonacci(2),
        fibonacci(3),
        fibonacci(4),
        fibonacci(5),
        fibonacci(6),
        fibonacci(7),
    )
}
```
The basic idea is that the top 128 bytes "belong" to the caller, while the
bottom 128 bytes belongs to the callee.  One DP frame that could be used is as
follows:
```
|   ...    | $82...
+----------+
| (result) | $80 (belongs to caller, filled in by callee)
+----------+
| input n  | $7E (belongs to callee, filled in by caller)
+----------+
| local a  | $7C
+----------+
| local b  | $7A
+----------+
|   ...    | $78...

```
To call this function, you'd need to invoke a sequence like the following:
```
TDC         ; 2 cycles
SEC         ; 2 cycles
SBC #4      ; 3 cycles
TCD         ; 2 cycles
LDA #n
STA $7E
JSR fibonacci
TDC         ; 2 cycles
INC         ; 2 cycles
INC         ; 2 cycles
TCD         ; 2 cycles
; total overhead: 17 cycles w/out parameters, 21+ with.
```

Adjusting D takes a minimum of 8 clock cycles either direction.  Each parameter
fill takes 4 cycles to store, plus however many to compute its value, which we
don't account for here.  The subroutine call overhead is 12 cycles (6 JSR, 6
RTS), but we also skip that since this will always be incurred no matter what.
We're looking at, assuming D is properly aligned, 17 clock cycles of frame
management overhead, not including parameter passing overhead; a minimum of 21
with.  This cost must be incurred with each subroutine call; however, the
function itself can be implemented with the assumption that D is set up
appropriately, and that it has free reign over its direct page.

Another disadvantage of this approach is that it cannot handle stacked
structures, or more than 128 bytes worth of locals and parameters at a time.
That size can be fudged a bit with some cleverness in how far you adjust the D
register on entry or exit from a function.  For most subroutines, this isn't a
problem.  It may become a problem with structure constructors, and will almost
certainly become a problem for functions which maintain local arrays of values.

Here's the same code for calling sum8:
```
; earlier in the code, the caller executes at least eight STA dp instructions,
; at 4 cycles each.  This totals 32 cycles on its own.
TDC
SEC
SBC #18
TDC
JSR sum8
TDC
CLC
ADC #18
TCD
; total overhead: 30 cycles w/out parameters, 62+ with.
```
Taken individually, these cycle counts are competitive with other alternatives
discussed here.  This is what nested function calls would look like though:
```
; fn nested() -> u16 { sum8(fib(0), fib(1), ..., fib(7)) }
;
; result of nested assigned to $80.
nested:
    ; result of sum8 at $80
    ; param 0 of sum8 at $7E
    ; result of fib $7E
    ; param 0 of fib at $7C
    STZ $7C
    TDC
    DEC
    DEC
    TCD
    JSR fib
    TDC
    INC
    INC
    TCD
    ; param 1 at $7C
    ; result of fib $7C
    ; param 0 of fib at $7A
    LDA #1
    STA $7A
    TDC
    SEC
    SBC #4
    TCD
    JSR fib
    TDC
    CLC
    ADC #4
    TCD
    ; ... and so on ...
    LDA #7
    STA $6E
    TDC
    SEC
    SBC #18
    TCD
    JSR fib
    TDC
    CLC
    ADC #18
    TCD
    ; finally we can compute the sum and return.
    JMP sum8
```

Notice how the placement of results in the direct page allows us to amortize
parameter setup overhead for the call to sum8.  Thus, the "overhead" for
calling sum8 is trivial, as the parameter passing overhead for all the calls to
fibonacci serve double-duty.  Admittedly, this is a pathological case; however,
this pattern appears frequently enough in programming that it's valuable to
optimize for.

Total clocks spent per call to fibonacci is 4 cycles to set parameter value, 9
cycles to decrement D, and 9 cycles again to restore D, totaling 22 cycles each
call.  There are eight of them, for a total of 176 cycles.  These cycles *also*
establish the frame for sum8, which means its overhead is effectively zero in
this case.  Thus, total overheads for nested here comes to only 176 cycles.

Alternative 2.  Using the D as a Frame Pointer on the Processor Stack.

If we use the calling convention pioneered by the Pascal compiler used to write
the GS/OS operating system for the Apple IIgs computer, we could overlap the
data and control stacks.  This allows us to use instructions like PER, PEI, and
PEA to establish parameters and local variables conveniently.  We would use the
D register as a frame pointer into the single system stack, which would allow
us to use direct page as a way of touching values on the stack.

To call the fibonacci subroutine, we need to reserve space for the result, then
push the parameter, and then make the call.
```
PEA 0   ; space for result, 5 cycles
PEA n   ; push argument, 5 cycles
JSR fibonacci
PLA     ; discard argument, 5 cycles
PLA     ; get result in A, 5 cycles
; total overhead: 20+ cycles (doesn't include locals overhead)

PEA 0   ; space for result, 4 cycles
PEA 1   ; argument; 4 cycles
PEA 2   ; argument; 4 cycles
PEA 3   ; argument; 4 cycles
PEA 4   ; argument; 4 cycles
PEA 5   ; argument; 4 cycles
PEA 6   ; argument; 4 cycles
PEA 7   ; argument; 4 cycles
PEA 8   ; argument; 4 cycles
JSR sum8
TSC     ; remove args, 2 cycles
CLC     ; 2 cycles
ADC #16 ; 3 cycles
TCS     ; 2 cycles
PLA     ; 5 cycles to recover result
; total overhead: 50+ cycles
```
So far so good; however, once inside the subroutine, we need to reserve space
for our local variables.  And, when we're done, we need to clean the results up
off the stack.
```
fibonacci:
    PEA 0       ; 5 cycles
    PEA 1       ; 5 cycles
L1: LDA 7,S
    BEQ L2
    LDA 1,S
    PHA         ; Remember to account for extra space on stack in offsets!
    CLC
    ADC 5,S
    STA 3,S
    PLA         ; OK, back to normal offsets here.
    STA 3,S
    DEC 7,S
    JMP L1
L2: LDA 1,S
    STA 9,S
    PLA         ; 5 cycles
    PLA         ; 5 cycles
    RTS

sum8:
    LDA 3,S
    CLC
    ADC 5,S
    ADC 7,S
    ADC 9,S
    ADC 11,S
    ADC 13,S
    ADC 15,S
    ADC 17,S
    STA 19,S
    RTS

nested:
    PEA 0       ; result for sum8; 5 cycles
    PEA 0       ; result for fib
    PEA 0
    JSR fib
    PLA         ; note: result already in place, so leave it on stack.

    PEA 0       ; 5 cycles
    PEA 1       ; 5 cycles
    JSR fib     ; 20 cycles
    PLA         ; 5 cycles

    PEA 0
    PEA 2
    JSR fib
    PLA
    ...
    PEA 0
    PEA 7
    JSR fib
    PLA

    JSR sum8
    TSC         ; 2 cycles
    CLC         ; 2 cycles
    ADC #16     ; 3 cycles
    TCS         ; 2 cycles

    ; propagate result into calling frame
    PLA         ; 5 cycles
    STA 3,S     ; 5 cycles
    RTS
```

By my count, the subroutine call overhead for fibonacci is 20 cycles not
accounting for parameters, 40 with parameters.  For sum8, however, no locals
are used, so it can run with only parameter setup overheads (50 cycles).

A call to nested seems to incur 10+8(35)+9+10=309 clock cycles of stack
management overhead.
```
PEA 0       ; 5 cycles of overhead
JSR nested  ; 299 cycles of overhead
PLA         ; 5 cycles of overhead
```


Alternative 3.  Direct Page as Pseudo-Registers.

This one is tough to characterize.  Inspired by calling conventions on RISC
processors due to their similar constraints.

Let us assign (say) sixteen direct-page pseudo-registers A0-A15 for procedure
argument passing, and (say) sixteen local variable direct-page pseudo-registers
(V0-V15).  Let's further use A0 for results.  The remainder of direct page
would be used as temporaries for use in a procedure.  Because these are
essentially dynamically-scoped variables, they would need to be preserved on
the stack if we needed to modify them for any reason.  Let's see how that
measures up.

```
nested:
    ; prolog code
    PEI A1          ; 6 cycles
    PEI A2          ; 6 cycles
    PEI A3          ; 6 cycles
    PEI A4          ; 6 cycles
    PEI A5          ; 6 cycles
    PEI A6          ; 6 cycles
    PEI A7          ; 6 cycles

    ; body
    LDA #7
    STA A0          ; 4 cycles
    JSR fibonacci
    LDA A0
    STA A7          ; 4 cycles

    LDA #6
    STA A0
    JSR fibonacci
    LDA A0
    STA A6

    ...

    STZ A0          ; 4 cycles
    JSR fibonacci
    JSR sum8

    ; epilog code
    PLA             ; 5 cycles each
    STA A7          ; 4 cycles each
    PLA
    STA A6
    ...
    PLA
    STA A0
    RTS

fibonacci:
    ; prolog code
    ;
    ; No need to save A0 since it is an output.
    ; No need to save A1-A7, as we don't modify them.
    ; We do, however, use two local variables.
    PEI V0      ; 6 cycles
    STZ V0
    PEI V1      ; 6 cycles
    LDA #1
    STA V1

    ; body
L1: LDA A0
    BEQ L2
    LDA V1
    PHA
    CLC
    ADC V0
    STA V1
    PLA
    STA V0
    DEC A0
    JMP L1

L2: LDA V1
    STA A0

    ; epilog code
    PLA         ; 5 cycles
    STA V1      ; 4 cycles
    PLA         ; 5 cycles
    STA V0      ; 4 cycles
    RTS

sum8:
    ; prolog code (none)
    ;
    ; We use eight arguments, but we don't modify them (except for A0, used to
    ; deliver the results).  We do not use any locals.  Therefore, no need to
    ; stack anything.

    ; body
    LDA A0
    CLC
    ADC A1
    ADC A2
    ADC A3
    ADC A4
    ADC A5
    ADC A6
    ADC A7
    STA A0
    ; epilog code
    RTS
```

For fibonacci, I count 34 cycles of overhead using this approach (4 for
argument setup, and 30 for stacking locals).  For sum8, total overhead comes to
zero.  But, that doesn't mean there is no overhead.  Note that the caller has
to preserve a large swath of pseudo-registers.  That's overhead; however, it is
amortized over *all* subroutine invokations.  Saving and restoring these values
costs 15 cycles each, and there are seven that need to be saved, totalling 105
cycles.  We call subroutines nine times within the nested function, so we can
distribute that cost to them for an amortized overhead of 11.6 cycles each
call.  Thus, each call to fibonacci incurs 45.6 cycles of overhead (thanks to
it needing to save two locals every time), while the call to sum8 has 11.6.
Total cycles spent in overhead is 8(45.6)+11.6=376.4.

If we could somehow convince the compiler that the locals may safely use
discardable temporaries (either through explicit syntax or through inference),
we can eliminate much of that overhead, and the total overhead for nested can
drop to 9(11.6)=104.4.  The problem is, the caller would need to know if its
own locals would interfere with the discardable locals of the subroutines; that
is, there would need to be some mechanism to communicate this either through
inference or through explicit syntax in the function signature.  With
conventional language designs, there's no way for that information to cross the
interface boundary without breaking the binary interface, even if all other
factors are perfectly abstracted.

So, on the one hand, this alternative is either the *fastest* or it is the
*slowest*, depending on how the code is structured and how intelligent the
compiler is.  It is easy to see, with this analysis, why this approach works so
well for RISC processors.

Discussion.

Alternative 1 has zero cost for locals allocation, since the *entire* local-
part of direct page is *assumed* to be free for use by the callee.  Only the D
pointer need be adjusted.  This allows it to fall below 40 cycles for
subroutine call overhead.

Alternative 2 surprises me.  It *should* be the case that it is slower than
alternative 1, but only because of the need to propagate values between frames
across control data like return addresses, but I can't see how that accounts
for a 2x performance impact.  That should be closer to a 1.00-and-change factor
at most.  Taking a single call into fibonacci, we see that its runtime overhead
comes to 40 cycles, exactly identical to alternative 1.

Yet, when we see fibonacci called in the context of another procedure, we see
that we rack up the overhead quite fast, coming to 309 clock cycles total.  I
speculate half of this overhead comes from needing to clean up the stack.  If
we look at the code for nested in alternative 1, we see that it's possible to
establish frames in just the right spots so that results of subroutines appear
in parameters for other subroutines that need to be called.  We need only
manipulate the D register once (although, in the code above, I do so *twice*
between calls to fibonacci) for each frame adjustment, thus saving a lot of
time.

Alternative 3 requires stacking every direct page location that the subroutine
uses *and which is not overtly a parameter or a disposable temporary*.
Potentially, that comes to zero items stacked.  It need only do this *once*
during the function's prolog and epilog code.  Countering this, the caller need
only stack those temporaries that it considers nondisposable.  This leads to
three classes of direct page variables:

1. Subroutine inputs and/or outputs (arguments).  Callee-saved as required to
   call subroutines of its own with impunity.

2. Nondisposable temporaries (local variables).  Callee-saved.

3. Disposable temporaries (intermediate computations, generally not programmer
   visible).  Caller-saved as required by the caller's needs.

This potentially can save a lot of time.  However, as we see above, it can also
cause large overheads because of the 65816's lack of an optimized "pop into
direct page" instruction.  The PLA/STA combination takes almost double the
amount of time a single PEI instruction takes.  If you need to call a
subroutine with any number of local variables frequently, the overhead of
stacking those locals' direct page values will add up fast.

Even with a theoretical counter-part to the 65816's PEI instruction, we don't
save enough to be able to compete with either of the two previous alternatives.
Access to the CPU stack is just plain *slow* no matter how you look at it.  A
single PEI instruction preserves a single direct-page location; however, an
adjustment to the D register takes 9 cycles at worst, and potentially grants
you access to 250 or more direct page locations.


## Decision

Support for local variables implies an application binary interface, and I'm
selecting alternative 1 for ROIL's ABI.  It might not *look* very fast when you
examine the code, and it might even be verbose as hell.  But, I feel it offers
a great compromise between verbosity and overall program performance, erring on
the latter.

## Consequences

Code can potentially be more verbose.  Prior to a subroutine call, one must
adjust D to point at the right location, which can take six bytes.  After the
procedure returns, D must be restored, which can also take upwards of six
bytes.  Plus the three or four to invoke the subroutine, and we're looking at
16 bytes per call-site in the worst cases.

In retrospect, and with no obvious performance hit that I can see, we can
reduce the code size like so:
```
PHD
TDC
SEC
SBC #nn
TCD
JSR theProcedure
PLD
```
We can do this because the PHD/PLD pair consumes 9 clock cycles.  The cost to
adjust D is also 9 cycles.  Thus, we still take 18 cycles to manage the D
register, but we can do so in only 11 bytes.

If we allocate up instead of allocating down like a stack, we can say that
direct page location $00 is defined to always hold the result, $02.. the
parameters, and some space above that your procedure's locals.  This is the
same method BCPL uses in its various virtual machine targets.  It would also
give a procedure much more space for locals.  On the other hand, it also would
give zero access to the caller's frames.  So, without an explicit back-pointer,
support for nested procedure scopes (a la Pascal or Rust) couldn't exist.  This
sounds like it'd be a valuable trade-off, as I almost never use this feature
(even in my Pascal and Rust code).  That said, if this *is* desirable in the
future, stacking D a la the call code above would allow us effortless access to
previous call frames using the 65816's (n,S),Y addressing mode.

