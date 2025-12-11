#include test2file.asm
printstring:
set r8 2
jmp printloop

printloop:
load r1 r2
eq r1 0
jz end
int r1
add r2 1
jmp printloop

end:
ret

main:
set r8 1
set r2 0
int 12
store 0 "h"
store 1 "a"
store 2 "l"
store 3 "l"
store 4 "o"
store 5 32
store 6 "w"
store 7 "o"
store 8 "r"
store 9 "l"
store 10 "d"
call printstring
set r8 2
int 10
hlt