@echo off
cd timer-test-source
rgbasm -L -o main.o main.asm
rgblink -o timer-test.gb main.o
rgbfix -v -p 0xFF .\timer-test.gb
cd ..
@echo on