x0=0
ra=1
sp=2
gp=3
a0=10
a1=11
a2=12
a3=13
a4=14
a5=15
a6=16
a7=17
t0=31

		jal	x0,_start

		align	8
hPrg:		dword	0

_start:		auipc	t0,0
		sd	a0,hPrg-_start(t0)	; Save hProg

		addi	a0,x0,$41
		addi	a7,x0,$2A
		ecall

		ld	a0,hPrg-_start(t0)
		addi	a7,x0,3		; attempt to close hProg
		ecall

		addi	a0,x0,$42
		addi	a7,x0,$2A
		ecall

		addi	a7,x0,0		; just return from the startup event
		ecall			; should never return

_loop:		addi	a0,x0,$43
		addi	a7,x0,$2A
		ecall
		jal	x0,_loop

