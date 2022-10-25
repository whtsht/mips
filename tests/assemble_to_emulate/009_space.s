.text
.globl main
main:
addi $v0, $0, 1

addi $t0, $0, A
addi $t1, $0, B

lw $t2, 0($t0)
sw $t2, 0($t1)

lw $t2, 16($t0)
sw $t2, 16($t1)


lw $a0, 0($t1)
syscall

lw $a0, 16($t1)
syscall


jr $ra

.data
A: .word 1, 2, 3, 4, 5
B: .space 20
