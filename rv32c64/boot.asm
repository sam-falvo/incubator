; vim:ts=16:sw=16:noet:ai: 
; xa -w -M boot.asm -o boot
;
;	Nucleus - RISC-V RV32I Virtual Machine Monitor
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

	; The nucleus is broken into two or three fragments, depending
	; on how big it becomes.  The first fragment is 4KB and resides
	; in RAM at $C000-$CFFF.  This fragment is always in memory,
	; regardless of which memory configuration is active.

	jsr LoadHAL
	jsr ConfigNucleus

	; If the nucleus ever begins to exceed 12KB in size, we have an
	; additional 4KB under $D000-$DFFF that we may use.

	; The second fragment is 8KB and resides in RAM at $E000-$FFFF,
	; underneath the KERNAL ROM.  This fragment is active only when
	; the CPU is running in native mode.  Thus, this fragment
	; contains all the code which is I/O device independent.
	; However, this fragment is important because it provides the
	; native-mode trap vectors.

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
	*=$C000

;=======================================================================
;	JSR ConfigKERNAL
;
; Prepare the address space for calling KERNAL procedures.  This switches
; KERNAL and BASIC ROM images into memory at $E000-$FFFF and $A000-$BFFF,
; respectively.  Additionally, I/O registers are exposed at $D000-$DFFF.
;
; Calling this function from the affected address ranges will yield
; unpredictable results, as the calling code will have been replaced by the
; time this routine returns.
;
; Preparation:
;   Registers:	none
;   Memory:	any configuration.
;   Calls:	none
;
; Results:
;   Registers:	none
;   Memory:	KERNAL configuration.
;=======================================================================

+ConfigKERNAL:	pha
	php
	_AS

	; Ensure data direction bits are correct.
	lda !0
	ora #$07
	sta !0

	; Switch in I/O, KERNAL, and BASIC.
	lda !1
	ora #$07
	sta !1

	plp
	pla
	rts

;=======================================================================
;	JSR ConfigNucleus
;
; Prepare the address space for calling Nucleus procedures.  This switches
; KERNAL and BASIC ROM images out of memory at $E000-$FFFF and $A000-$BFFF,
; respectively.  Additionally, RAM is exposed at $D000-$DFFF.
;
; Preparation:
;   Registers:	none
;   Memory:	any configuration.
;   Calls:	none
;
; Results:
;   Registers:	none
;   Memory:	Nucleus configuration.
;=======================================================================

; Prepare the address space for calling Nucleus procedures.
+ConfigNucleus:	pha
	php
	_AS

	; Ensure data direction bits are correct.
	lda !0
	ora #$07
	sta !0

	; Switch out I/O, KERNAL, and BASIC.
	lda !1
	and #$F8
	sta !1

	plp
	pla
	rts


;=======================================================================
;	JSR myCHROUT
;
; Writes the character in A to the current output device via the KERNAL
; CHROUT/BSOUT procedure.
;
; Preparation:
;   Registers:	A = character (bits 0-7 only).
;   Memory:	Nucleus configuration.
;   Calls:	Same as CHROUT.
;
; Results:
;   Registers:	A = error code if any.
;   Flags:	C set if error.
;   Memory:	STATUS, RSSTAT updated
;
;=======================================================================

+myCHROUT:	phx
	phy
	php
	_AXL
	sta tmpA

	jsr ConfigKERNAL
	tsa
	sta progSP
	lda kernSP
	tas
	lda #$0000
	tad
	sec
	xce
	cli
	.as
	.xs

	lda tmpA
	jsr CHROUT

	sei
	clc
	xce
	_AXL
	tsa
	sta kernSP
	lda progSP
	tas
	lda #NDP
	tad
	jsr ConfigNucleus
	plp
	ply
	plx
	rts
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
	*=$E000

AGAIN:	ldx #MSG
	_AS
AGN:	lda !0,x
	beq DONE
	phx
	jsr myCHROUT
	plx
	inx
	jmp AGN
DONE:	jmp AGAIN

MSG:	.byte "HELLO FROM 65816!  ",0
.)
LenE000	= *-$E000

	*=ModuleStart+LenE000

