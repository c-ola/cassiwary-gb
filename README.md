to compile for sm83: sdcc -msm83 test/main.c -o test/build/

for binary file: makebin -p < test/build/main.ihx > test/main.bin

hexdump: xxd main.bin
