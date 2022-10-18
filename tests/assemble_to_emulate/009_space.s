addi $t0, $0, L

lw $a0, 0($t0)
addi $v0, $0, 1
syscall

jr $ra

.data
A: .word 1, 2, 3, 4, 5
B: .word 20
