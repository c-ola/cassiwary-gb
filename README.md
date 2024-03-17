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
- 02-interrupts
- 03-op sp, hl
- 04-op r, imm
- 05-op rp
- 06-ld r, r
- 08-misc instrs 
- 09-op r, r
- 10-bit ops
- 11-op a, (hl)


Failed:
- 07-jumps : just goes to narnia, STOPS, and then breaks when it gets resumed
jr, jp, call, ret, rst, 
