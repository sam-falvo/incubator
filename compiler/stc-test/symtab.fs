.(   Symbol Table ) cr


\ This module associates an internal name to an external name.
\ Internal and external names may be identical.
\ Internal names correspond to their Forth names; thus, punctuation is allowed.
\ External names are valid ca65 assembler symbol names.


2048 constant #Symbols

#Symbols cells constant /Vector

create Keys
  /Vector allot

create Values
  /Vector allot

: SymTab0
  Space0
  Keys /Vector -1 fill
  Values /Vector -1 fill ;

SymTab0


variable slen
variable skey
variable xqofs

: -match ( ofs -- ofs )
  dup Keys + @
    dup -1 = if drop exit then
  Recall skey @ slen @ compare 0= if xqofs ! 1 r> drop then ;

: match? ( caddr u -- f )
  slen ! skey !
  0 begin dup /Vector < while -match cell+ repeat drop
  0 ;

: E002
  abort" E002: Symbol already defined" ;

: undefined ( caddr u - caddr u )
  2dup match? E002 ;

\ An "xref" is a symbol S which refers to an external procedure also named S.
\ Thus, both the key and the value are equal.
\ Unlike Forth, the assembler's namespace is not hyperstatic.
\ Thus, duplicate symbol names are explicitly an error.

: E004
  abort" E004: Symbol table full." ;

: filled ( k ofs -- k ofs ) \ if vector element is filled
  dup Keys + @ -1 xor if exit then
\ filled ( k ofs -- ofs )     otherwise
  swap over Keys + !  r> drop ;

: key! ( k -- ofs )
  0 begin dup /Vector < while filled cell+ repeat E004 ;

: val! ( v ofs -- )
  Values + ! ;

: xref ( caddr u -- )
  undefined  Intern dup key! val! ;

\ An "xname" is a symbol S which refers to an external procedure
\ named N.  In every other respect, it's just like an "xref".

: xname ( N[caddr u] S[caddr u] -- )
  undefined  Intern -rot Intern swap key! val! ;


: E005
  ." E005: "  skey @ slen @ type cr
  abort" E005: Symbol not defined." ;

\ Recall a value for a given search key.
: xquery ( caddr u -- caddr' u' )
  match? if xqofs @ Values + @ Recall exit then
  -1 E005 ;


: name ( -- caddr u )
  32 parse ;

: check ( caddr u -- caddr u )
  dup 0= abort" E006: Symbol expected." ;


\ Define a list of word names as xrefs; the list ends at
\ the end of the line.  Note that comments are NOT recognized.
: xref:
  begin name dup if xref else 2drop exit then again ;


\ Define a single xname.
: xname:
  name check name check xname ;



marker passed

: TestMatch?Undef
  SymTab0  S" foo" match?  depth 1 xor abort" TestMatch?Undef: stack"
                           abort" TestMatch?Undef: seeing things" ;

TestMatch?Undef

: TestXref
  SymTab0 S" foo" xref  depth 0 xor abort" TestXref: stack"
  S" bar" match? abort" TextXref: seeing things"
  S" foo" match? 0= abort" TextXref: blind" ;

TestXref

: TestXquery
  SymTab0 S" plus" S" +" xname  S" minus" S" -" xname  S" foo" xref
  S" foo" xquery S" foo" compare abort" TestXquery: foo mismatch"
  S" +" xquery S" plus" compare abort" TestXquery: + mismatch"
  S" -" xquery S" minus" compare abort" TestXquery: - mismatch" ;

TestXquery

passed

