addi $a0, $zero, 0
addi $t0, $zero, 10

L:

addi $v0, $0, 1
syscall

addi $a0, $a0, 1

slt $t1, $a0, $t0
bne $t1, $zero, L

jr $ra
