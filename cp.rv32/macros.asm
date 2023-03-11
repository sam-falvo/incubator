; vim:ts=16:sw=16:noet:ai
;
;	Nucleus - RISC-V RV32I Virtual Machine Monitor
;	Copyright 2023 Samuel A. Falvo II
;

#define bge		bcs
#define blt		bcc
#define tas		tcs
#define tsa		tsc
#define tad		tcd
#define tda		tdc

#define _AL		rep #$20:.al
#define _AS		sep #$20:.as
#define _XL		rep #$10:.xl
#define _XS		sep #$10:.xs
#define _AXL		rep #$30:.al:.xl
#define _AXS		sep #$30:.as:.xs

; Symbol names are, conventionally, 14 characters or less so that they can fit
; on a single assembly language line.
;
;       |------------|
#define Breakpoint	.word $FF00
