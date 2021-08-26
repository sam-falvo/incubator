.(   Utilities ) cr


\ Answer with the next name from the current input stream.
\ If no name is available, return with a 0-length string.
: name ( -- caddr u )
  32 parse ;

\ Check to make sure that a name actually exists.
\ If not, abort with an error.
: check ( caddr u -- caddr u )
  dup 0= abort" E006: Symbol expected." ;

