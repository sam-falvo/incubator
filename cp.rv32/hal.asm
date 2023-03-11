; vim:ts=16:sw=16:noet:ai: 
;
;	Hardware Abstraction Layer
;
;	RISC-V RV32I Virtual Machine Monitor
;	Copyright 2023 Samuel A. Falvo II
;

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

;=======================================================================
;	JSR PrintInline
;	.BYT "Hello world",13,0
;
; Prints a string encoded after the JSR to the current output device
; using myCHROUT.
;
; Errors are ignored.
;
; Preparation:
;   Registers:	A must be 16-bits wide.
;   Memory:	Nucleus configuration.
;   Calls:	Same as CHROUT.  Must be calling from bank 0.
;
; Results:
;   Registers:	A used.
;   Flags:	none.
;   Memory:	STATUS, RSSTAT updated
;
;=======================================================================

+PrintInline:	phx
	phy
	php
.(
	_AXL
	lda 6,s
	tax
	_AS
again:	lda !1,x	; X=ptr-1, so use 1,X to get next byte.
	beq onNull
	jsr myCHROUT
	inx
	jmp again

onNull:	_AL
	inx	; Skip NUL byte.
	txa
	sta 6,s	; Skip past embedded string data on RTS.

	plp
	ply
	plx
	rts
.)

;=======================================================================
;	JSR myCHRIN
;
; Reads a character from the current input device and returns it in the
; low byte of A.  See the C64 PRG for more details on CHRIN/BASIN.
; CHROUT/BSOUT procedure.
;
; Preparation:
;   Registers:	none.
;   Memory:	Nucleus configuration.
;   Calls:	Same as CHRIN.
;
; Results:
;   Registers:	A = character or error code.
;   Flags:	C set if error.
;   Memory:	STATUS, RSSTAT updated
;
;=======================================================================

+myCHRIN:	phx
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
	jsr CHRIN
	sta tmpA

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
	_AS
	lda tmpA
	plp
	ply
	plx
	rts

;=======================================================================
;	JSR OutputBuffer
;
; Preparation:
;   Registers:	X/Y in 16-bit mode.
;	X = address of buffer (bank 0) to read bytes from.
;	Y = number of bytes to output.
;   Memory:	none.
;   Flags:	none.
;   Calls:	CKOUT (to set current device to write to)
;
; Results:
;   Registers:	A, X, Y used.
;   Memory:	device dependent.
;   Flags:	none.
;=======================================================================

+OutputBuffer:	php
	_AS
	.xl
.(
	cpy #0
	beq onDone

again:	lda !0,x
	jsr myCHROUT
	inx
	dey
	bne again
onDone:
.)
	plp
	rts
