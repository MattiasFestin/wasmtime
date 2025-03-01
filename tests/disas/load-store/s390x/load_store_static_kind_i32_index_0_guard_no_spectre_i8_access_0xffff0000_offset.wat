;;! target = "s390x"
;;! test = "compile"
;;! flags = " -C cranelift-enable-heap-access-spectre-mitigation=false -O static-memory-forced -O static-memory-guard-size=0 -O dynamic-memory-guard-size=0"

;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
;; !!! GENERATED BY 'make-load-store-tests.sh' DO NOT EDIT !!!
;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

(module
  (memory i32 1)

  (func (export "do_store") (param i32 i32)
    local.get 0
    local.get 1
    i32.store8 offset=0xffff0000)

  (func (export "do_load") (param i32) (result i32)
    local.get 0
    i32.load8_u offset=0xffff0000))

;; wasm[0]::function[0]:
;;       lg      %r1, 8(%r2)
;;       lg      %r1, 0x10(%r1)
;;       la      %r1, 0xa0(%r1)
;;       clgrtle %r15, %r1
;;       stmg    %r14, %r15, 0x70(%r15)
;;       lgr     %r1, %r15
;;       aghi    %r15, -0xa0
;;       stg     %r1, 0(%r15)
;;       llgfr   %r7, %r4
;;       clgfi   %r7, 0xffff
;;       jgh     0x34
;;       ag      %r7, 0x58(%r2)
;;       llilh   %r4, 0xffff
;;       stc     %r5, 0(%r4, %r7)
;;       lmg     %r14, %r15, 0x110(%r15)
;;       br      %r14
;;
;; wasm[0]::function[1]:
;;       lg      %r1, 8(%r2)
;;       lg      %r1, 0x10(%r1)
;;       la      %r1, 0xa0(%r1)
;;       clgrtle %r15, %r1
;;       stmg    %r14, %r15, 0x70(%r15)
;;       lgr     %r1, %r15
;;       aghi    %r15, -0xa0
;;       stg     %r1, 0(%r15)
;;       llgfr   %r7, %r4
;;       clgfi   %r7, 0xffff
;;       jgh     0x84
;;       ag      %r7, 0x58(%r2)
;;       llilh   %r4, 0xffff
;;       llc     %r2, 0(%r4, %r7)
;;       lmg     %r14, %r15, 0x110(%r15)
;;       br      %r14
