to compile for sm83: sdcc -msm83 test/main.c -o test/build/
to compile for z80 gb: sdcc -mgbz80 test/main.c -o test/build/

for binary file: makebin -p < test/build/main.ihx > test/main.bin
for binary file: makebin -Z < test/build/main.ihx > test/main.gb

hexdump: xxd main.bin
