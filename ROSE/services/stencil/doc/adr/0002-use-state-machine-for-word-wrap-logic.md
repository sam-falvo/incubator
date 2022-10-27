# 2. use state machine for word wrap logic

Date: 2022-10-27

## Status

Tentative

## Context

I tried writing the logic to implement decent word-wrap, only to code myself into a corner every time.
In this attempt, I try to use a subset of the Cleanroom Engineering process to arrive at what I believe is a reasonable solution.

## Decision

STIMULUS:

R	Reset
C	Any non-breaking character (incl. whitespace)
B	any breaking character (typ. whitespace)
L	Line-break (typ. carriage return, paragraph break, etc.)
M	Margin breach (generated internally)
E	End of input file


SEQUENCE ENUMERATION:

R	h=0;V.clear;start=end=brk=0
C	impossible
B	impossible
L	impossible
M	impossible
E	impossible

RR	same as R
RC	h+=width;end+=1
RB	h+=width;end+=1;brk=end
RL	h=0;V.push(start,end+1);start=end+1
RM	impossible
RE	terminal

RCR	same as R
RCC	h+=width;end+=1.  same as RC
RCB	h+=width;end+=1;brk=end.  same as RB
RCL	h=0;V.push(start,end+1);start=end+1.  same as RL
RCM	V.push(start,end-1);start=end-1;h=0. // No breaking space yet, so line-wrap
RCE	V.push(start,end).  terminal

RBR	same as R
RBC	h+=width;end+=1.
RBB	h+=width;end+=1;brk=end.  same as RB
RBL	h=0;V.push(start,end+1);start=end+1.  same as RL
RBM	same as RB.
RBE	V.push(start,end).  terminal

RLR	same as R
RLC	h+=width;end+=1.  same as RC
RLB	h+=width;end+=1;brk=end.  same as RB
RLL	h=0;V.push(start,end+1);start=end+1.  same as RL
RLM	impossible
RLE	terminal

RCMR	same as R
RCMC	h+=width;end+=1.  same as RC
RCMB	h+=width;end+=1;brk=end.  same as RB
RCML	h=0;V.push(start,end+1);start=end+1.  same as RL
RCMM	same as RCM
RCME	V.push(start,end).  terminal

RBCR	same as R
RBCC	h+=width;end+=1.  same as RBC
RBCB	h+=width;end+=1;brk=end.  same as RB
RBCL	h=0;V.push(start,end+1);start=end+1.  same as RL
RBCM	V.push(start,brk+1);start=brk+1;h=0.  same as RC
RBCE	V.push(start,end).  terminal


Canonical	State
Sequence	Variable	Before	After		Comments
R		V		-	empty
		head		-	0
		start		-	0
		end		-	0
		brk		-	0

RC		head		h	h+width(c)
		end		e	e+1

RB		head		h	h+width(b)
		end		e	e+1
		brk		b	e

RL		head		h	0
		V		v	v,(s,e+1)	Include CR/LF/PP char
		start		s	e+1

RCM		V		v	v,(s,e-1)	e-1 b/c C belongs on next line
		start		s	e-1		No brk space yet, so line wrap
		head		h	0

RCE		V		v	v,(s,e)		Terminal state.

RBC		head		h	h+width(c)
		end		e	e+1

RBE		V		v	v,(s,e)		Terminal state.

RLE							Terminal state.

RCME		V		v	v,(s,e)		Terminal state.

RBCE		V		v	v,(s,e)		Terminal state.



## Consequences

I executed the process somewhat wrong;
that's OK though, I am confident it yields the correct results anyway.

