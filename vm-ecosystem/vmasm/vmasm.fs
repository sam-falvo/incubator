( These work for GForth under Linux.  You'll probably need to adapt these for your platform. )

262144 CONSTANT /tImage
CREATE tImage
  /tImage ALLOT

: >real ( addr - addr' )   tImage + ;
: >img  ( addr' - addr )   tImage - ;


VARIABLE tH
: ORG ( n - )       tH ! ;
: THERE ( - a )     tH @ ;
: TALLOT ( n - )    tH +! ;
: TALIGN ( - )      tH @ 3 + -4 AND tH ! ;


: B! ( n addr - )   >real C! ;
: H! ( n addr - )   2DUP B! SWAP 8 RSHIFT SWAP 1 + B! ;
: W! ( n addr - )   2DUP H! SWAP 16 RSHIFT SWAP 2 + H! ;
: D! ( n addr - )   2DUP W! SWAP 32 RSHIFT SWAP 4 + W! ;

: B@ ( addr - n )   >real C@ ;
: H@ ( addr - n )   DUP B@ SWAP 1+ B@ 8 LSHIFT OR ;
: W@ ( addr - n )   DUP H@ SWAP 2 + H@ 16 LSHIFT OR ;
: D@ ( addr - n )   DUP W@ SWAP 4 + W@ 32 LSHIFT OR ;

: B, ( n -- ) THERE B!   1 TALLOT ;
: H, ( n -- ) THERE H!   2 TALLOT ;
: W, ( n -- ) THERE W!   4 TALLOT ;
: D, ( n -- ) THERE D!   8 TALLOT ;


: BINARY   2 BASE ! ;


( RISC-V Integer Registers )

: xreg ( n -- n+1 )   DUP CONSTANT DUP CONSTANT 1+ ;

0
  xreg  x0  zero
  xreg  x1  ra
  xreg  x2  sp
  xreg  x3  gp
  xreg  x4  tp
  xreg  x5  t0
  xreg  x6  t1
  xreg  x7  t2

  xreg  x8  s0
  xreg  x9  s1
  xreg x10  a0
  xreg x11  a1
  xreg x12  a2
  xreg x13  a3
  xreg x14  a4
  xreg x15  a5

  xreg x16  a6
  xreg x17  a7
  xreg x18  s2
  xreg x19  s3
  xreg x20  s4
  xreg x21  s5
  xreg x22  s6
  xreg x23  s7

  xreg x24  s8
  xreg x25  s9
  xreg x26  s10
  xreg x27  s11
  xreg x28  t3
  xreg x29  t4
  xreg x30  t5
  xreg x31  t6
DROP


( Immediate Fields )

  ( U-type displacements )
: bit20 ( n -- n' )      20 RSHIFT    1 AND ;
: bits19-12 ( n -- n' )  12 RSHIFT $0FF AND ;
: bit11 ( n -- n' )      11 RSHIFT    1 AND ;
: bits10-1 ( n -- n' )    1 RSHIFT $3FF AND ;

  ( B-type displacements )
: bit12 ( n -- n' )      12 RSHIFT    1 AND ;
: bit11 ( n -- n' )      11 RSHIFT    1 AND ;
: bits10-5 ( n -- n' )    5 RSHIFT $03F AND ;
: bits4-1 ( n -- n' )     1 RSHIFT $00F AND ;

  ( B-type immediates )
: bits11-5 ( n -- n' )    5 RSHIFT $07F AND ;
: bits4-0 ( n -- n' )              $01F AND ;


( Instruction Packing and Forms )

: FN6<< ( n -- n' )  63 AND 26 LSHIFT ;
: FN7<< ( n -- n' )  127 AND 25 LSHIFT ;
: RS2<< ( n -- n' )  31 AND 20 LSHIFT ;
: RS1<< ( n -- n' )  31 AND 15 LSHIFT ;
: FN3<< ( n -- n' )  7 AND 12 LSHIFT ;
: RD<< ( n -- n' )   31 AND 7 LSHIFT ;
: OP<< ( n -- n' )   127 AND ;


: Uimm ( n -- n' )
  $FFFFF000 AND ;

: typeU, ( imm rd opc -- )
  OP<< SWAP RD<< OR SWAP UImm OR W, ;


: Jdisp ( n -- n' )
  >R R@ bit20 19 LSHIFT
     R@ bits10-1 9 LSHIFT OR
     R@ bit11 8 LSHIFT OR
     R> bits19-12 OR
  12 LSHIFT ;

: typeJ, ( disp rd opc -- )
  OP<< SWAP RD<< OR SWAP JDisp OR W, ;


: Iimm ( n -- n' )
  $0FFF AND 20 LSHIFT ;

: Ishamt6 ( n -- n' )
  63 AND 20 LSHIFT ;

: Ishamt5 ( n -- n' )
  31 AND 20 LSHIFT ;

: Bdisp ( n -- n' )
  >R R@ bit12 31 LSHIFT
     R@ bits10-5 25 LSHIFT OR
     R@ bits4-1 8 LSHIFT OR
     R> bit11 7 LSHIFT OR ;

: Bimm ( n -- n' )
  >R R@ bits11-5 25 LSHIFT
     R> bits4-0 7 LSHIFT OR ;

: typeI, ( imm rs1 rd fn3 opc -- )
  OP<< SWAP FN3<< OR SWAP RD<< OR SWAP RS1<< OR SWAP Iimm OR W, ;

: typeIsh5, ( shamt rs1 rd fn7 fn3 opc -- )
  OP<< SWAP FN3<< OR SWAP FN7<< OR SWAP RD<< OR SWAP RS1<< OR SWAP Ishamt5 OR W, ;

: typeIsh6, ( shamt rs1 rd fn6 fn3 opc -- )
  OP<< SWAP FN3<< OR SWAP FN6<< OR SWAP RD<< OR SWAP RS1<< OR SWAP Ishamt6 OR W, ;

: typeB, ( disp13 rs1 rs2 fn3 opc -- )
  OP<< SWAP FN3<< OR SWAP RS2<< OR RS1<< OR Bdisp OR W, ;

: typeBs, ( imm12 rs1 rs2 fn3 opc -- )
  OP<< SWAP FN3<< OR SWAP RS2<< OR RS1<< OR Bimm OR W, ;

: typeR, ( rs1 rs2 rd fn7 fn3 opc -- )
  OP<< SWAP FN3<< OR SWAP FN7<< OR SWAP RD<< OR SWAP RS2<< OR SWAP RS1<< OR W, ;


BINARY

: lui, ( n xr - )             0110111 typeU, ;
: auipc, ( n xr - )           0010111 typeU, ;

: jal, ( disp21 xr - )        1101111 typeJ, ;

: jalr, ( imm12 rs1 rd - )    000 1100111 typeI, ;

: beq, ( disp13 rs1 rs2 - )   000 1100011 typeB, ;
: bne,                        001 1100011 typeB, ;
\ ----                        010 1100011 typeB, ;
\ ----                        011 1100011 typeB, ;
: blt,                        100 1100011 typeB, ;
: bge,                        101 1100011 typeB, ;
: bltu,                       110 1100011 typeB, ;
: bgeu,                       111 1100011 typeB, ;

: lb, ( imm12 rs1 rd - )      000 0000011 typeI, ;
: lh,                         001 0000011 typeI, ;
: lw,                         010 0000011 typeI, ;
: ld,                         011 0000011 typeI, ;
: lbu,                        100 0000011 typeI, ;
: lhu,                        101 0000011 typeI, ;
: lwu,                        110 0000011 typeI, ;
: ldu,                        111 0000011 typeI, ;

: sb, ( imm12 rs1 rs2 - )     000 0100011 typeBs, ;
: sh,                         001 0100011 typeBs, ;
: sw,                         010 0100011 typeBs, ;
: sd,                         011 0100011 typeBs, ;

: addi, ( imm12 rs1 rd - )    000 0010011 typeI, ;
: slti,                       010 0010011 typeI, ;
: sltiu,                      011 0010011 typeI, ;
: xori,                       100 0010011 typeI, ;
: ori,                        110 0010011 typeI, ;
: andi,                       111 0010011 typeI, ;

: slli, ( shamt rs1 rd - )    000000 001 0010011 typeIsh6, ;
: srli,                       000000 101 0010011 typeIsh6, ;
: srai,                       010000 101 0010011 typeIsh6, ;

: add, ( rs1 rs2 rd - )       0000000 000 0010011 typeR, ;
: sub,                        0100000 000 0010011 typeR, ;
: slt,                        0000000 010 0010011 typeR, ;
: sltu,                       0000000 011 0010011 typeR, ;
: xor,                        0000000 100 0010011 typeR, ;
: or,                         0000000 110 0010011 typeR, ;
: and,                        0000000 111 0010011 typeR, ;

: sll, ( rs1 rs2 rd - )       0000000 001 0010011 typeR, ;
: srl,                        0000000 101 0010011 typeR, ;
: sra,                        0100000 101 0010011 typeR, ;

DECIMAL

: fence, ( fm pred succ rs1 rd -- )
  >R >R 15 AND 20 LSHIFT SWAP
        15 AND 24 LSHIFT OR SWAP
        15 AND 28 LSHIFT OR
  R> R> 0 15 typeI, ;

: fence.tso,                  8 3 3 0 0 fence, ;
: pause,                      0 1 0 0 0 fence, ;

: ecall, ( - )
  0 ( imm ) 0 ( rs1 ) 0 ( rd ) 0 $73 typeI, ;

: ebreak, ( - )
  1         0         0        0 $73 typeI, ;

BINARY

\ RV64I-specifics

: addiw, ( imm rs1 rd - )     000 0011011 typeI, ;

: slliw, ( shamt rs1 rd - )   0000000 001 0011011 typeIsh5, ;
: srliw,                      0000000 101 0011011 typeIsh5, ;
: sraiw,                      0100000 101 0011011 typeIsh5, ;

: addw, ( rs1 rs2 rd - )      0000000 000 0111011 typeR, ;
: subw,                       0100000 000 0111011 typeR, ;
: sllw,                       0000000 001 0111011 typeR, ;
: srlw,                       0000000 101 0111011 typeR, ;
: sraw,                       0100000 101 0111011 typeR, ;

: fence.i, ( imm rs1 rd - )   001 0001111 typeI, ;

: csrrw, ( csr rs1 rd - )     001 1110011 typeI, ;
: csrrs,                      010 1110011 typeI, ;
: csrrc,                      011 1110011 typeI, ;
: csrrwi, ( csr imm5 rd - )   101 1110011 typeI, ;
: csrrsi,                     110 1110011 typeI, ;
: csrrci,                     111 1110011 typeI, ;

DECIMAL


\ Convenient Constructs and Macros

: quote, ( xr -- aJAL )
  ( Jumps AHEAD, except sets xr to point at code immediately following. )
  THERE SWAP 0 SWAP jal, ;

: mergeU ( disp addr -- )
  SWAP Jdisp OVER ( addr disp addr )
  W@ ( addr disp JAL )
  OR ( addr JAL' - this works as long as operand of JAL is 0 )
  SWAP W! ;

: unquote, ( aJAL xr -- )
  ( Resolves prior quote, and sets xr to length of block spanned. )
  OVER THERE SWAP - 4 - ( aJAL xr len; 4 accounts for JAL insn )
  TALIGN >R >R ( aJAL || xr len )
  THERE OVER - ( aJAL disp || xr len )
  OVER mergeU ( || xr len )
  R> R> ( xr len )
  OVER IF
    SWAP x0 SWAP ori,
  ELSE
    2DROP
  THEN ;

: defer, ( - )
  CREATE 0 quote, , ( JAL X0,... )
  DOES>  @ THERE - ra jal, ;

: is, ( a - )
  ' >BODY @ SWAP OVER ( aJAL a aJAL )
  - DUP ." displacement = " . cr ( aJAL disp )
  SWAP mergeU ;

