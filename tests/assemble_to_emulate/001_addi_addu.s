addi $t0, $zero, 2 
addi $t1, $zero, 3
addu $a0, $t0, $t1

# syscall: print integer
addi $v0, $0, 1
syscall
jr $31
