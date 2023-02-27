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
