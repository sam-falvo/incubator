; vim:ts=16:sw=16:noet:ai: 
; xa -w -M nucleus.asm -o nucleus
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

	sei
	clc
	xce
	_AXL
	lda #NDP
	tad
	tsa
	sta kernSP
	lda #NSTK
	tas

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

myCHROUT:	php
	_AXL
	sta tmpA

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
	plp
	rts


MSG:	.byte "HELLO FROM 65816!  ",0

