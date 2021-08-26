.(   Colon Compiler ) cr


: header ( caddr u -- )
  2dup xref
  ." enter_" type ." :" cr ;

: call ( caddr u -- )
  ."   jsr enter_" type cr ;

\ Attempt to parse another name/lexeme from the current input.
\ If not possible, try to refill the input buffer and try again.
\ If *that* doesn't work, then just return end of input via a
\ 0-length string.
: lexeme ( -- caddr u )
  name dup if exit then
  2drop refill if name exit then
  0 0 ;


create template
  0 c, 'c c, 'm c, 'p c, ': c,
  32 allot

: -compiler ( caddr u -- caddr u [if not a compiler word] [otherwise] )
  2dup 32 min template 5 + swap move
  dup 32 min 4 + template c!
  \ R> drops the return address for token.
  template find if nip nip r> drop execute else drop then ;

: token ( caddr u -- )
  -compiler call ;


: T:
  name check header
  begin lexeme dup if token then again ;
 

: cmp:exit
  ."   rts" cr ;

: cmp:;
  \ First r> drops return address for -compiler.
  \ Second r> drops return address for T:, thus ending
  \ the colon interpreter.
  cmp:exit r> r> 2drop ;

