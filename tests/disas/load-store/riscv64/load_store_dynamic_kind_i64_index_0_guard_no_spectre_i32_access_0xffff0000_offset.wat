;;! target = "riscv64"
;;! test = "compile"
;;! flags = " -C cranelift-enable-heap-access-spectre-mitigation=false -W memory64 -O static-memory-maximum-size=0 -O static-memory-guard-size=0 -O dynamic-memory-guard-size=0"

;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
;; !!! GENERATED BY 'make-load-store-tests.sh' DO NOT EDIT !!!
;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

(module
  (memory i64 1)

  (func (export "do_store") (param i64 i32)
    local.get 0
    local.get 1
    i32.store offset=0xffff0000)

  (func (export "do_load") (param i64) (result i32)
    local.get 0
    i32.load offset=0xffff0000))

;; wasm[0]::function[0]:
;;       addi    sp, sp, -0x10
;;       sd      ra, 8(sp)
;;       sd      s0, 0(sp)
;;       mv      s0, sp
;;       lui     a5, 0x3fffc
;;       addi    a1, a5, 1
;;       slli    a4, a1, 2
;;       add     a1, a2, a4
;;       bgeu    a1, a2, 8
;;       .byte   0x00, 0x00, 0x00, 0x00
;;       ld      a4, 0x60(a0)
;;       bgeu    a4, a1, 8
;;       .byte   0x00, 0x00, 0x00, 0x00
;;       ld      a4, 0x58(a0)
;;       add     a4, a4, a2
;;       lui     a2, 0xffff
;;       slli    a5, a2, 4
;;       add     a4, a4, a5
;;       sw      a3, 0(a4)
;;       ld      ra, 8(sp)
;;       ld      s0, 0(sp)
;;       addi    sp, sp, 0x10
;;       ret
;;
;; wasm[0]::function[1]:
;;       addi    sp, sp, -0x10
;;       sd      ra, 8(sp)
;;       sd      s0, 0(sp)
;;       mv      s0, sp
;;       lui     a5, 0x3fffc
;;       addi    a1, a5, 1
;;       slli    a3, a1, 2
;;       add     a1, a2, a3
;;       bgeu    a1, a2, 8
;;       .byte   0x00, 0x00, 0x00, 0x00
;;       ld      a3, 0x60(a0)
;;       bgeu    a3, a1, 8
;;       .byte   0x00, 0x00, 0x00, 0x00
;;       ld      a3, 0x58(a0)
;;       add     a3, a3, a2
;;       lui     a2, 0xffff
;;       slli    a4, a2, 4
;;       add     a3, a3, a4
;;       lw      a0, 0(a3)
;;       ld      ra, 8(sp)
;;       ld      s0, 0(sp)
;;       addi    sp, sp, 0x10
;;       ret
