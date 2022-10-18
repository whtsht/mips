addi $t0, $0, L

# Load word
# dest: $a0 offset: 0, address: L 
lw $a0, 0($t0)
addi $v0, $0, 1
syscall

jr $ra

.data
L: .word 100
