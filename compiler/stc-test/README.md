**This project and its README are works in progress.**

# Abstract
# Introduction
When I started working on the Kestrel-1 prototype back in 2004,
I built it using a series of Radio Shack solderless breadboards,
a W65C816 processor running at 4MHz,
and a whopping 32KB of RAM mapped into the upper half of bank 0,
with I/O occupying the lower-half.
The I/O consisted of a single W65C22 VIA chip.
System software could be loaded into it via a specialized "IPL circuit",
which allowed a PC to boot the system via its parallel port.
The idea was to use this system as a springboard for more refined, more complete homebrew computer systems.
One of my first ideas was to create a kind of 65816-based Jupiter ACE:
when you first turned the machine on, the idea was to drop you into a Forth environment.
For various reasons, I never got the chance to fulfill my initial vision.

Fast forward to 2021, and I have come to the realization that I only have several more years until its 20th anniversary.
It would sure be nice to be able to complete the computer design by then,
as a kind of treat to myself and to others who'd love to play with it too.
However, it's hard enough to bring up new hardware from scratch;
it would be great if creating the first system software image for it could contribute as little as possible to this challenge.

{List Contributions Here}

# The Problem (1pp)
Writing a Forth environment from scratch is hard work.
It is laborious and error-prone,
thanks in large part to the large number of inter-linked data structures comprising the dictionary.
And, if at a later time you decide you want to alter those structures,
you typically must restart your project from scratch, with precious little code reused.

# The Idea (2pp)
Ideally, I'd like to write the Forth compiler in Forth itself.
This suggests creating a *metacompiler*.
It would take in a source file containing Forth code and
it would produce a corresponding assembly language file.
Using the `ca65` assembler and corresponding linker,
we can then produce a firmware or boot-disk image that the hardware can run.

# The Details (5pp)


    
# Related Work (1-2pp)
# Conclusion/Further Work (0.5-1.0pp)
# References
[1] Wirth, Niklaus.  _Compiler Construction_.  2017 May.  Accessed 2021 August 16.  Part 1: http://people.inf.ethz.ch/wirth/CompilerConstruction/CompilerConstruction1.pdf
[2] Wirth, Niklaus.  _Compiler Construction_.  2017 May.  Accessed 2021 August 16.  Part 2: http://people.inf.ethz.ch/wirth/CompilerConstruction/CompilerConstruction2.pdf
