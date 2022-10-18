addi $v0, $zero, 1

addi $t0, $zero, 4
addi $t1, $zero, 5
mult $t1, $t0
mflo $a0
syscall

jr $ra
