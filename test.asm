#include test2file.asm
eq:
set r8 2
set r7 74
int r7
jmp end

neq:
set r8 2
set r7 78
int r7
jmp end

end:
set r8 0
int r7

main:
set r1 50
set r2 20
neq r1 r2
jz eq
jnz neq