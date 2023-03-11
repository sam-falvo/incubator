# Compiler

A silly project which I'll almost certainly never complete.

As its name implies, it is intended to be a compiler for an as-yet unspecified programming language.
I will refine the language specification as it evolves.

I'm currently targeting the Z80 and 65816.


## Why?

Just Because.

It's always been a curiosity of mind to find out what goes into writing a "real" compiler,
something a bit more sophisticated than a Forth dialect.

Like I said, this isn't likely to be finished.
Nor, for that matter, do I expect anyone else to ever adopt it.
(Unless the language ends up evolving into a dialect of C.  But, even then, there's `sdcc`, etc.)

The 65816 port has a critical bug: zero page location $00 is reused without
preservation, which can create opportunities for data corruption.  But, for the
simple test cases I've tested so far, things seem to work OK.  But, if code
breaks in the future for what looks like an inexplicable reason, this is a good
candidate root cause.  Too lazy to fix this at the moment.

## Strategy

* Incremental, test-driven approach.

* Destination-Driven Code Generation.  This is a HUGE WIN, but combining this with delayed code generation from Wirth would make a very potent pair.

* Parse s-expressions to start with, create alternative syntax later if required.  (It's not required.)
