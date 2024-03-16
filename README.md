## Arguments

Log memory to a file 
```sh
--log-memory #-m
```

Path of rom to run with the emulator
```sh
--rom-path <PATH_TO_ROM> #-p
```

Help
```sh
--help #-h
```

## Blarggs Tests
Passed:
- 01-special
- 04-op r, imm
- 05-op rp
- 06-ld r, r
- 08-misc instrs 
- 10-bit ops


Failed:
- 02-interrupts : Timer dont work
- 03-op sp, hl : weird outputs in terminal (could use to figure out whats happening)
- 07-jumps : completely failed (jumped to narnia and got stuck somewhere probably), no LCD output
- 09-op r, r : 2F 88 89 8A 8B 8C 8D 8F 98 99 9A 9B 9C 9D 9F failed
- 11-op a, (hl) : 8E 9E failed
