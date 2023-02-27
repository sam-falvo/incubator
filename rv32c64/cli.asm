; vim:ts=16:sw=16:noet:ai: 
;
;	Command Line Interface
;
;	RISC-V RV32I Virtual Machine Monitor
;	Copyright 2023 Samuel A. Falvo II
;

	*=$E000

;=======================================================================
;	JSR EnterCLI
;
; Launches the command-line interface for the virtual machine monitor.
;
; Preparation:
;   Registers:	none.
;   Memory:	Nucleus configuration.
;   Flags:	none.
;   Calls:	none.
;
; Results:
;   Registers:	A, X, Y used.
;   Memory:	none.
;   Flags:	none.
;
; NOTE:	This procedure does not return.
;=======================================================================

+EnterCLI:	jsr PrintInline
	.byt 13,13,"   **** RISC-V RV32I EMULATOR V1 ****   ",13,0
.(
again:	jsr PrintInline
	.byt "- ",0
	jsr InputCmd
	jsr InterpretCmd
	jmp again
.)

;=======================================================================
;	JSR InputCmd
;
; Reads a line of input from the user into the CmdLineBuffer.
;
; Preparation:
;   Registers:	none.
;   Memory:	none.
;   Flags:	none.
;   Calls:	CHKIN (usually to reset input device to console)
;
; Results:
;   Registers:	X, Y used.
;   Memory:	CmdLineLength set to number of bytes read, which
;	may include the final CR if it fits in the buffer.
;	CmdLineBuffer contains the command text read.
;   Flags:	none.
;=======================================================================

+InputCmd:	php
	_AXS
.(
	jsr ClearCLBuffer
	ldx #0
again:	jsr myCHRIN
	cpx #CmdLineCapacity
	bge onFull
	sta CmdLineBuf,x
	inx
onFull:	cmp #CR
	beq onReturn
	cmp #$00
	beq onReturn
	jmp again
onReturn:	stx CmdLineLength
	stz CmdLineLength+1
.)
	plp
	rts

;=======================================================================
;	JSR ClearCLBuffer
;
; Prepares the command line buffer for input.
;
; Preparation:
;   Registers:	none.
;   Memory:	none.
;   Flags:	none.
;   Calls:	none.
;
; Results:
;   Registers:	A, X used.
;   Memory:	CmdLineLength set to 0.  CmdLineBuffer set to all NULs.
;   Flags:	none.
;=======================================================================

+ClearCLBuffer:	php
	_AXS
.(
	stz CmdLineLength
	stz CmdLineLength+1

	ldx #0
	txa
again:	sta CmdLineBuf,x
	inx
	cpx #CmdLineCapacity
	bne again
.)
	plp
	rts

;=======================================================================
;	JSR InterpretCmd
;
; Interprets the command-line stored in CmdLineBuffer.
;
; Preparation:
;   Registers:	none.
;   Memory:	CmdLineBuffer contains CmdLineLength bytes of text.
;   Flags:	none.
;   Calls:	none.
;
; Results:
;   Registers:	A, X, Y used.
;   Memory:	depends on command executed.
;   Flags:	depends on command executed (convention is none).
;=======================================================================

+InterpretCmd:	php
	_AXL

	jsr PrintInline
	.byt 13,"YOU TYPED:",13,"  ",0

	ldx #CmdLineBuf
	ldy CmdLineLength
	jsr OutputBuffer

	jsr PrintInline
	.byt 13,0

	plp
	rts
