INCLUDE "hardware.inc"

SECTION "Header", ROM0[$100]

  jp EntryPoint

  ds $150 - @, 0 ; Make room for the header

EntryPoint:
    ld a, 0
    ld hl, $C000

CopyLoop:
    ld [hl], a
    inc a
    inc l
    
    cp $20
    jp nz, CopyLoop

DMA:
    ld a, $00
    ld [rHDMA1], a
    ld [rHDMA3], a
    ld [rHDMA4], a
    ld a, $C0
    ld [rHDMA2], a
    ld a, $01
    ld [rHDMA5], a

    
    ld d, $00
    ld a, $0
    ld hl, $C000

CheckLoop:
    cp [hl]
    jp nz, Failed
    inc a
    inc l
    
    cp $20
    jp nz, CheckLoop

Done:
    jp Done

Failed:
    ld d, $FF
    jp Done