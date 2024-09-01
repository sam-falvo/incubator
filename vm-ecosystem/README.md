This VM Ecosystem Project is designed to support running a unique operating system environment.
This VM and OS combination is based in concept on the UXN virtual machine environment;
however, there are some notable differences.

## Program Structure

UXN applications consist exclusively of event handlers.
The "start" handler is hard-coded to the address 0100h (256 decimal);
therefore, all applications have an entry point at this address.
All other event handlers are configured dynamically
via special device I/O ports called *vectors*.
For example, the *screen vector* is invoked 60 times per second (basically, emulating an NTSC vertical retrace period).
The *console vector* is invoked whenever a new character arrives on `stdin`.
Etc.
When a handler has completed its task,
it "returns" back to the Uxn emulator with the `BRK` instruction.

VM/OS applications behave similarly.
The "start" handler is hard-coded to the address 01000h (4096 decimal).
As with UXN,
all other handlers are configured dynamically via a set of `ecall`s.
Although the RISC-V instruction set has an `ebreak` instruction,
it is intended for use exclusively with debuggers.
Thus, to return from an event handler back to the VM/OS environment,
we use a specific `ecall` reserved just for that purpose.

## Memory Layout

UXN's memory layout is astonishingly simple:
it simply consists of 65536 bytes of RAM.
(Since it's all RAM, calling UXN programs "ROMs" is a bit of a misnomer.
There's a reason for it, but it's beyond the scope of this document.)
All memory locations are accessible and mapped.

Under VM/OS,
the 64-bit address space of the RV64I instruction specification
clearly would be impractical to implement in the same way.
Thus, only specific portions of the address space are mapped and accessible.

RAM goes from 01000h to 0FFFFFh, giving the program room for almost 1MiB of capacity.
There *may or may not* be resources mapped in higher blocks of memory,
depending on what resources are currently in use.
Attempting to access a region of unmapped memory will cause a fatal trap.

## Unmapped Regions

UXN software never traps:
every opcode is defined,
and every memory location is mapped.
Though programs start at 0100h,
it is not uncommon to place application state
starting at address 0000h (the "zero" page).

VM/OS relies on the RISC-V instruction set architecture.
Under VM/OS,
the memory from 00000h-00FFFh will cause the processor to trap,
leading to termination of the program.
This is intended to help catch null pointer dereferences.
Likewise, attempting to access unmapped memory regions will also cause a trap to occur.
Opcodes are 32-bits wide;
since it's impossible for a processor to support 4.2 *billion* opcodes,
it follows that some instructions are undefined.
Thus, undefined instructions are also trapped.
Finally, attempting to execute instructions with a misaligned program counter will also cause a trap.

## Stacks vs Registers

The UXN virtual machine additionally provides programs with an evaluation stack and a return stack.
Both of these stacks consist of 256 bytes.
No boundaries exist; pushing a two-byte value at location 255 will place the high byte at 255 and the low byte at 0.

VM/OS applications are designed for the RISC-V instruction set.
Thus, its state is kept in its registers: X1-X31.

## System Services

UXN applications exploit system services through a virtual machine-specific set of devices.
Each device offers 16 "ports" through which reads and writes can not only affect state changes,
but also accesses to these ports may cause (desirable) side-effects.
For example, one may cause the *screen device* to display a sprite somewhere on the display by first
setting the X/Y coordinates of where to draw the sprite,
followed by writing the sprite's draw mode and memory address.
As soon as the memory address is written, the Varvara emulator will place the screen on display.

VM/OS has similar features;
however, they are called by the `ecall` instruction.
Registers A0-A3 are configured with parameters as-needed;
A7 then contains a function code to invoke `ecall` with.
The emulator looks at A7, then dispatches to a handler appropriate for the number provided.

## Graphic Displays

UXN images can depend on a two layer, four-color display.

VM/OS images depend on a single layer, monochrome display.
Higher color depths may be possible depending on the specific VM implementation;
but, monochrome is the only one *guaranteed* to be supported universally.

