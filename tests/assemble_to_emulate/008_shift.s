addi $v0, $0, 1
addi $t0, $zero, -8

sll  $a0, $t0, 2
syscall

srl  $a0, $t0, 2
syscall

jr $ra
