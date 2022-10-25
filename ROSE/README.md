# 2022-10-25

I just had a flash of inspiration.
I've long been looking for a good name for my graphical environment projects.
I've finally settled on one.

As of today,
this project will henceforth be known as ROSE,
the Retro-inspired Operating System Environment.
It is, to some extent, a pun on the name GEOS.
And, while running under a hosted OS, it's clearly not an OS unto itself,
there's little (if anything?) preventing it from being hosted on bare metal.

For fun,
I am entertaining the idea of naming a word processor for it AmbROSE.
If anyone asks,
I'll just say that I named it after Ambrose Bierce.

I haven't decided if it's time to graduate the project to a self-standing repository yet.

# 2022-10-24

Working on apps and system services together has yielded a graphics API that I quite enjoy using.
In particular, I am especially chuffed with the API for printing text to a stencil (bitmap).

```rust
let mut p = SimplePrinter::new(
	&mut stencil,
	((L, T), (R, B)),
	&font
);
p.print(&a_string);
```

Seems like a simple thing,
but this properly wraps text around the righthand margin
and stops printing if the text falls off the bottom edge.
This is literally the API I've *always* wanted for every GUI environment I've coded for.
The *only* one which seems to implement anything like this is GEOS for the Commodore 64.
(Not sure about PC/GEOS; considering its lineage, though, I suspect it did.)

This is plenty good enough for the task of creating a slide deck player.
However, it's not good enough for a general purpose text reader or editor yet.
I will need to support word-wrapping at the very least,
and eventually, support for multiple fonts and styles.

But if all you're doing is throwing up a dialog box or rendering a menu or status line,
SimplePrinter is your go-to API.
Nothing else is simpler.

# 2022-09-30

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

