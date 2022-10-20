.text
.globl main
main:

addi $v0, $0, 1

ori $t1, $zero, 10
ori $t2, $zero, -20

add $a0, $t1, $t2

syscall


jr $ra
