.(   Utilities ) cr

\ Answers the address of the currently referenced byte in the
\ input buffer.
: pt ( -- caddr )
  source drop >in @ + ;

\ Answers the length of the input buffer.
: lim ( -- u )
  source nip ;

\ Answers true if >IN is at the end of input buffer.
: eoi ( -- f )
  >in @ lim >= ;

\ Terminates ws and -ws if end of input buffer reached.
: -eoi
  eoi if r> drop then ;

\ Terminates ws if whitespace is no longer found.
: sp
  pt c@ [char] ! u>= if r> drop then ;

\ Terminates -ws if whitespace is found.
: -sp
  pt c@ [char] ! u< if r> drop then ;

\ Advances >in by a single character.
: eat
  1 >in +! ;

\ Skip whitespace in the current input buffer.
\ Skip non-whitespace in the current input buffer.
\ In either case, never go beyond the end of the
\ input buffer.
\ 
\ These are not Unicode aware.  But, for UTF-8
\ encoded sources, I don't foresee any compatibility
\ issues.

: ws
  begin -eoi sp eat again ;

: -ws
  begin -eoi -sp eat again ;

\ Answer with the next name from the current input stream.
\ If no name is available, return with a 0-length string.
\ Unlike ANSI Forth's PARSE word, this word considers anything
\ below codepoint $21 to be whitespace.  Thus, tabs, newlines,
\ spaces, carriage returns, etc. are all considered whitespace.
: name ( -- caddr u )
  ws pt -ws pt eat over - ;

\ Check to make sure that a name actually exists.
\ If not, abort with an error.
: check ( caddr u -- caddr u )
  dup 0= abort" E006: Symbol expected." ;

