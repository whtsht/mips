addi $t0, $0, L
lw $a0, 0($t0)
addi $v0, $0, 1
syscall

jr $ra

.data
L: .word 100
