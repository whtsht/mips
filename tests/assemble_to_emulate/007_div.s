addi $v0, $0, 1

addi $t0, $zero, 26
addi $t1, $zero, 4

div $t0, $t1

mflo $a0
syscall

mfhi $a0
syscall

jr $ra
