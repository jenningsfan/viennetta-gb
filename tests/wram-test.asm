INCLUDE "hardware.inc"

SECTION "Header", ROM0[$100]

  jp EntryPoint

  ds $150 - @, 0 ; Make room for the header

EntryPoint:
  ld c, 1
  ld d, 0

Load_WRAM:
  ld a, c
  ld [rSVBK], a

  add a, $AB
  ld [$D000], a
  inc c
  
  ld a, c
  sub a, 8
  jp nz, Load_WRAM

  ld c, 1
Check_WRAM:
  ld a, c
  ld [rSVBK], a

  ld a, c
  add a, $AB
  ld b, a
  ld a, [$D000]
  sub a, b
  jp nz, Failed

  inc c
  
  ld a, c
  sub a, 8
  jp nz, Check_WRAM

Done:
  jp Done

Failed:
  ld d, 0xFF
  jp Done