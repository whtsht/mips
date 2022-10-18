j L

# The instruction below does not run.
addi $a0, $0, 34

L:
addi $a0, $0, -34

addi $v0, $0, 1
syscall
jr $ra
