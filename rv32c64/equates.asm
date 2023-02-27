; vim:ts=16:sw=16:noet:ai
;
;	Nucleus - RISC-V RV32I Virtual Machine Monitor
;	Copyright 2023 Samuel A. Falvo II
;

; Emulator Memory Map

BASORG	= $0801	; Start of BASIC program listing
NDP	= $0900	; Native Direct Page address
NSTK	= $0AFF	; Native Stack Bottom
tmpA	= $0B00	; Native/Emulation register linkage
kernSP	= $0B02	; Last known SP in KERNAL context
progSP	= $0B04	; Last known SP in nucleus context
CmdLineLength	= $0B06	; Length of command in buffer
CmdLineBuf	= $0B80	; 128-byte buffer for holding command line input
ORIGIN	= $1000	; Start of Nucleus text

; Handy Constants

CR	= $0D	; Carriage return

CmdLineCapacity	= 128	; Maximum length of command line buffer

; KERNAL routines.  These MUST be called in emulation mode only.

CHROUT	= $FFD2
CHRIN	= $FFCF
