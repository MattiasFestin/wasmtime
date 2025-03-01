;;! target = "aarch64"
;;! test = "winch"

(module
    (func (result i32)
	(i32.const 0x80000000)
	(i32.const -1)
	(i32.div_s)
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
;;       mov     x16, #0x80000000
;;       mov     w1, w16
;;       cbz     w0, #0x64
;;   34: cmn     w0, #1
;;       ccmp    w1, #1, #0, eq
;;       b.vs    #0x68
;;   40: sxtw    x0, w0
;;       sxtw    x1, w1
;;       sdiv    x1, x1, x0
;;       mov     w0, w1
;;       add     x28, x28, #0x10
;;       mov     sp, x28
;;       mov     sp, x28
;;       ldp     x29, x30, [sp], #0x10
;;       ret
;;   64: .byte   0x1f, 0xc1, 0x00, 0x00
;;   68: .byte   0x1f, 0xc1, 0x00, 0x00
