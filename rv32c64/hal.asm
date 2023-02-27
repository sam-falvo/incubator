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
