This directory holds programs which explore the idea of building productivity tools.

One of the first things I typically do
when I construct a homebrew computer
is build a slide presentation program for it,
so that I may actually use my homebrew design
to give a talk about it.

A slideshow program is an example of a productivity application.
So, too, would be some of my ideas for a word processor,
or desktop publisher,
or structured drawing program.
In fact, there is a lot of overlap between a slideshow player and a structured drawing program;
namely, the existence of an editor.

In a perfect world, the software that I write here
would be equally at home running under Linux or within a Commodore 64 or 128 emulator.
I would like to use Rust to do so.

Support under Linux would be trivial; just use the platform's default Rust compiler toolchain.
Rust, however, does not support the 6502/65816 processor architecture;
therefore, my approach would be to cross-compile to the RV32I instruction set.
*Maybe,* RV32IM.
But, definitely no more advanced than that.
Then, write a simple RV32I(M) emulator for the 65816,
and get it working with a SuperCPU 64 and/or ForthBox under emulation.
The SuperCPU at 20MHz should be able to offer very respectable performance.
The ForthBox, maybe, not so much; since it only runs at 4MHz.
It should at least be usable as a technology demonstrator, however.

This also means that the software would need to support very diverse filing systems.
Commodore DOS is optimized for sequential file access in a single, flat namespace,
while Linux is POSIX-compliant, with all that that implies.
If we want to be able to use GEOS fonts (which I'd recommend, since I really don't feel like making my own),
we'll need to support VLIR files as well.

The graphics output stack will also need to accomodate a monochrome 320x200 display as its smallest supported resolution.

Applicaton keyboard input would need to work well on the C64's 64-key keyboard as well.

For Linux support, I figure we can just use SDL2.
I've written enough GUI-like things using SDL2, even from Rust, to know that this technique should work well.

