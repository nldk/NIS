#include test2file.asm
main:
    ; Print 'H' (ASCII 72) using int code 2
    set r8 2
    int 72

    ; Print 'i' (ASCII 105)
    int 105

    ; Print newline (ASCII 10)
    int 10

    ; Allocate 5 slots of memory (int code 1)
    set r8 1
    int 5

    ; Store value 123 at memory address 0
    set r1 123
    store 0 r1

    ; Load value from address 0 back into r2
    load r2 0

    ; Print the number in r2 (int code 3)
    mov r7 r2
    call printnum

    ; Exit program (int code 0)
    set r8 0
    int 0