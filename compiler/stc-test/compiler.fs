only forth definitions

vocabulary metacompiler
  also metacompiler definitions

.( Compiling target metacompiler: ) cr

include utilities.fs
include strings.fs
include symtab.fs
include colon.fs

\ Clean up any garbage left over from unit tests.
Space0 SymTab0

