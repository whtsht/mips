# Mips Emulator

## File Header Format

```
Entry point (32bit)
Start point of text section (32bit)
Start point of data section (32bit)
```

## Support Instruction

| Name                   | Mnemonic | Opcode | Function | Type |
|------------------------|----------|--------|----------|------|
| Jump                   | j        | 0x2    | -        | J    |
| Jump Register          | jr       | 0x0    | 0x8      | R    |
| Add Unsigned           | addu     | 0x0    | 0x21     | R    |
| Sub Unsigned           | subu     | 0x0    | 0x23     | R    |
| And                    | and      | 0x0    | 0x24     | R    |
| Or                     | or       | 0x0    | 0x25     | R    |
| Add Immediate          | addi     | 0x8    | 0x0      | I    |
| Add Immediate Unsigned | addiu    | 0x9    | 0x0      | I    |
| Load Word              | lw       | 0x23   | 0x0      | I    |
| Store Word             | sw       | 0x2b   | 0x0      | I    |
| Branch On Equal        | beq      | 0x4    | -        | I    |
| Branch On Not Equal    | bne      | 0x5    | -        | I    |
| System Call            | syscall  | 0x0    | 0xc      | R    |
