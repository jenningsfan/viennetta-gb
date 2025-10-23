rgbasm -o wram-test.o wram-test.asm
rgblink -o wram-test.gb wram-test.o
rgbfix -C -v -p 0xFF ./wram-test.gb

rgbasm -o vram_dma.o vram_dma.asm
rgblink -o vram_dma.gb vram_dma.o
rgbfix -C -v -p 0xFF ./vram_dma.gb