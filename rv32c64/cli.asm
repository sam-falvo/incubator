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
;   Registers:	none.
;   Memory:	none.
;   Flags:	none.
;   Calls:	none.
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

+InputCmd:	rts

+InterpretCmd:	jmp InterpretCmd
