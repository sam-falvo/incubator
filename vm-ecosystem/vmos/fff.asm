x0		= 0
ra		= 1
sp		= 2
gp		= 3
a0		= 10
a1		= 11
a2		= 12
a3		= 13
a4		= 14
a5		= 15
a6		= 16
a7		= 17
t0		= 31

ecGetAttributes	= $0001
ecSetAttributes	= $0002
ecClose		= $0003

ecDbgOut	= $002A

		jal	x0,_start

		align	8
hPrg:		dword	0
resultCode:	dword	153

_start:		auipc	t0,0
		sd	a0,hPrg-_start(t0)	; Save hProg

		addi	a0,x0,$41
		addi	a7,x0,ecDbgOut
		ecall

		ld	a0,hPrg-_start(t0)	; Set return code and request to exit
		addi	a1,x0,1
_x:		auipc	a2,0
		addi	a2,a2,resultCode-_x
		addi	a7,x0,ecSetAttributes
		ecall

		ld	a0,hPrg-_start(t0)	; Close program handle
		addi	a7,x0,ecClose
		ecall

		addi	a0,x0,$42
		addi	a7,x0,ecDbgOut
		ecall

		addi	a7,x0,0		; just return from the startup event
		ecall			; should never return

_loop:		addi	a0,x0,$43
		addi	a7,x0,ecDbgOut
		ecall
		jal	x0,_loop

