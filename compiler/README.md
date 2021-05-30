# Compiler

A silly project which I'll almost certainly never complete.

As its name implies, it is intended to be a compiler for an as-yet unspecified programming language.
I will refine the language specification as it evolves.

I'm currently targeting the Z80.


## Why?

Just Because.

It's always been a curiosity of mind to find out what goes into writing a "real" compiler,
something a bit more sophisticated than a Forth dialect.

Like I said, this isn't likely to be finished.
Nor, for that matter, do I expect anyone else to ever adopt it.
(Unless the language ends up evolving into a dialect of C.  But, even then, there's `sdcc`, etc.)


## Strategy

* Incremental, test-driven approach.

* Destination-Driven Code Generation.

* Parse s-expressions to start with, create alternative syntax later if required.

