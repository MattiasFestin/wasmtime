;;! target = "aarch64"
;;! test = "winch"

(module
    (func (result i32)
        (i32.const -1)
	(i32.const 1)
	(i32.mul)
     )
)
;; wasm[0]::function[0]:
;;       stp     x29, x30, [sp, #-0x10]!
;;       mov     x29, sp
;;       mov     x28, sp
;;       mov     x9, x0
;;       sub     x28, x28, #0x10
;;       mov     sp, x28
;;       stur    x0, [x28, #8]
;;       stur    x1, [x28]
;;       orr     x16, xzr, #0xffffffff
;;       mov     w0, w16
;;       mov     x16, #1
;;       mul     w0, w0, w16
;;       add     x28, x28, #0x10
;;       mov     sp, x28
;;       mov     sp, x28
;;       ldp     x29, x30, [sp], #0x10
;;       ret
