; vim:ts=16:sw=16:noet:ai: 
; xa -w -M boot.asm -o boot
;
;	Bootstrap - RISC-V RV32I Virtual Machine Monitor
;	Copyright 2023 Samuel A. Falvo II
;

#include "macros.asm"
#include "equates.asm"

	; Program "header"

	.word BASORG	; load address

	; BASIC bootstrap

	*=BASORG

	.word NXTLIN	; Pointer to next BASIC program line.
	.word $000A	; Line 10
	.byte $9E	; "SYS" token ID
	.byte "4096"	; Parameter to SYS (must match ORIGIN)
	.byte $00	; End of line delimiter.
NXTLIN:	.word $0000	; No more BASIC lines.

	; Program text

	.dsb ORIGIN-*,$AA

	*=ORIGIN
	.as
	.xs

	; Turn off interrupts, and switch to 65816-native mode.

	sei
	clc
	xce
	_AXL

	; Move our stack and direct page so we don't conflict with
	; CBM's KERNAL.
	;
	; First, relocate direct page.

	lda #NDP
	tad

	; Next, relocate the stack to a new page.  Preserve our old
	; stack pointer first, though, so we can restore it when making
	; KERNAL calls.

	tsa
	sta kernSP
	lda #NSTK
	tas

	; The nucleus is broken into two fragments, depending on how
	; big it becomes.  The first fragment is 4KB and resides in RAM
	; at $C000-$CFFF.  This fragment is always in memory,
	; regardless of which memory configuration is active.  This
	; makes it ideal for hardware-related interfaces.

	jsr LoadHAL
	jsr ConfigNucleus

	; The second fragment is 8KB and resides in RAM at $E000-$FFFF,
	; underneath the KERNAL ROM.  This fragment is active only when
	; the CPU is running in native mode.  Thus, this fragment
	; contains all the code which is I/O device independent.
	; However, this fragment is important because it provides the
	; native-mode trap vectors.
	;
	; If the nucleus ever begins to exceed 12KB in size, we have an
	; additional 4KB under $D000-$DFFF that we may use to grow to
	; 16KB.

	jsr LoadNucleus
	jmp $E000

;=======================================================================
;	JSR LoadHAL
;
; Load the (at most) 4KB Hardware Abstraction Layer into memory at
; $C000.
;
; Preparation:
;   Registers:	A, X/Y must be 16-bits wide.
;   Memory:	KERNAL configuration.
;   Calls:	none
;
; Results:
;   Registers:	A, X, Y used.
;   Memory:	Nucleus HAL resides at $C000-$CFFF.
;
; NOTE:	Calling HAL functions before calling LoadHAL will
;	crash (you'll be executing uninitialized memory).
;=======================================================================

LoadHAL:	.al
	.xl
	ldx #BeginC000
	ldy #$C000
	lda #LenC000-1
	phb
	mvn 0,0
	plb
	rts

-ModuleStart	= *
BeginC000:
.(
#include "hal.asm"
.)
LenC000	= *-$C000
	*=ModuleStart+LenC000

;=======================================================================
;	JSR LoadNucleus
;
; Load the (at most) 8KB Emulator Nucleus into memory at $E000.
;
; Preparation:
;   Registers:	A, X/Y must be 16-bits wide.
;   Memory:	Any configuration.
;   Calls:	none.
;
; Results:
;   Registers:	A, X, Y used.
;   Memory:	Nucleus core resides at $E000-$FFFF.
;
; NOTE:	If the nucleus grows beyond 8KB, this procedure may
;	be extended to load a 12KB image starting at $D000.
;=======================================================================

LoadNucleus:	.al
	.xl
	ldx #BeginE000
	ldy #$E000
	lda #LenE000-1
	phb
	mvn 0,0
	plb
	rts

-ModuleStart	= *
BeginE000:
.(
#include "cli.asm"
.)
LenE000	= *-$E000

	*=ModuleStart+LenE000

