\ Tell compiler about externally provided primitives.
\ For each primitive P, reference to one of these names
\ will emit a JSR enter_P instruction.
\ 
\ Each word defined with primitives: could have been
\ defined using extern: N N (for any name N) as well.

xref: drop dup over swap and zgo nzgo go

\ Some primitive names don't map to a corresponding name in
\ the assembly language file.  Thus, they must be mangled
\ or otherwise renamed.  So, for example, a reference to
\ + would emit a JSR enter_plus instruction.

xname: cfetch c@
xname: cstore c!
xname: plus +
xname: minus -

: 2drop		drop drop ;
: 1-		1 - ;
: 1+		1 + ;

: emit		$FE20 c! ;
: key?		$FE21 c@ 1 and ;

: key		begin key? until $FE20 c@ ;

: cr		13 emit 10 emit ;
: type		begin dup while over c@ emit 1- swap 1+ swap repeat 2drop ;

: halt		begin key drop again ;
: hello-world   S" Hello world!" type cr halt ;

