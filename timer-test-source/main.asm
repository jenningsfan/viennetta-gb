; From https://gbdev.io/gb-asm-tutorial/part1/hello_world.html

INCLUDE "hardware.inc"

SECTION "Header", ROM0[$100]

  jp EntryPoint

  ds $150 - @, 0 ; Make room for the header

EntryPoint:
  ld a, 1
  ld [rTMA], a
  ld a, %100
  ld [rTAC], a
  ld a, 0
  ld [rTIMA], a
WaitVBlank:
  ; ld a, [rLY]
  ld a, [rTIMA]
  jp WaitVBlank
