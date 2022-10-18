addi $a0, $zero, 0
addi $t0, $zero, 10

# Repeat 10 times.
L:

addi $v0, $0, 1
syscall

addi $a0, $a0, 1
bne $a0, $t0, L

jr $ra
