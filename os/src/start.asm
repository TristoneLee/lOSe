.section .text
.global _start
_start:
        la sp, 0x80000000
        li a0, 4096
        csrr a1, mhartid
        addi a1, a1, 1
        mul a0, a0, a1
        add sp, sp, a0
        call rust_start
