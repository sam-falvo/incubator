: bit31   31 rshift 1 and ;
: bits30-21  21 rshift 1023 and ;
: bit20   20 rshift 1 and ;
: decodeJdisp ( n - n' )
  >R R@ bit31 20 lshift
     R@ bits30-21 1 lshift or
     R@ bit20 11 lshift or
     R> bits19-12 12 lshift or ;

