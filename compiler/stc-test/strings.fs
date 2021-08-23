.(   String storage ) cr


\ This module provides a place to put strings as they're
\ encountered in the program.  Strings are always appended;
\ they're never deleted.
\ 
\ Strings are stored one after another, each is prefixed
\ with a single byte indicating length of the string.
\ 
\ 5 'h 'e 'l 'l 'o 2 'c '@ ...
\ | \______ _____/ | \_ _/
\ |        V       |   V
\ |        |       |   |
\ `--------'       `---'


8192 constant /Space
create Space
  /Space allot

\ Every byte in Space that lies behind the barrier
\ is allocated (e.g., bytes 0..Barrier-1 are allocated).
\ 0 <= Barrier < /Space
variable Barrier

: Space0
  Space /Space 0 fill
  0 Barrier ! ;

Space0


\ Reserve space in the buffer.

: reserve ( n -- )
  Barrier +! ;

\ Store a character in the space buffer.

: where ( -- a )
  Barrier @ Space + ;

: sc, ( n -- )
  where c!  1 reserve ;

\ Remembering a string places the string into the
\ buffer and advances the barrier.

: remember ( caddr u -- )
  dup sc,  dup -rot  where swap move  reserve ;

\ Intern-ing a string causes us to remember it.
\ We place the string into the Space buffer, then
\ return the offset in which it appears.  That offset
\ then uniquely identifies that string for the rest
\ the program's run.

: E001
  abort" E001: Insufficient string storage space." ;

: roomy ( caddr u -- caddr u )
  dup 1+ Barrier @ + /Space U> E001 ;

: Intern ( caddr u -- u' )
  roomy  Barrier @ >r  remember  r> ;

\ Recalling a string produces a (caddr u) pair for the
\ specified intern.  The parameter MUST have been returned
\ by Intern; if it's not, you will receive garbage as a
\ result.

: E003
  abort" E003: Intern value out of range." ;

: valid ( u' -- u' )
  dup /Space U>= E003 ;

: Recall ( u' -- caddr u )
  valid  Space + count ;


marker passed

variable s1
variable s2
variable s3

: TestPlacement
  Space0  S" ABC" Intern drop
  Space C@ 3 xor abort" TestPlacement: length mismatch"
  Space 1 + C@ [char] A xor abort" TestPlacement: A mismatch"
  Space 2 + c@ [char] B xor abort" TestPlacement: B mismatch"
  Space 3 + c@ [char] C xor abort" TestPlacement: C mismatch"
  Barrier @ 4 xor abort" TestPlacement: Barrier mismatch" ;

TestPlacement

: TestIntern
  Space0 S" ABC" Intern s1 ! S" DEFGHI" Intern s2 !
  s2 @ s1 @ - 4 xor abort" TestIntern: Result mismatch"
  S" Forth" Intern s3 !
  s3 @ s2 @ - 7 xor abort" TestIntern: Result mismatch" ;

TestIntern

: TestRecall
  Space0 S" ABC" Intern s1 !  s1 @ Recall S" ABC" compare
  abort" TestRecall: Result mismatch" ;

TestRecall

passed

