;; Simple WASM module for demonstration
;; This is the source for the embedded WASM bytes in main.ts
;; To compile: wat2wasm simple.wat -o simple.wasm

(module
  ;; Memory: 1 page (64KB)
  (memory (export "memory") 1)

  ;; add: adds two i32 values
  (func (export "add") (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.add
  )

  ;; multiply: multiplies two i32 values
  (func (export "multiply") (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.mul
  )

  ;; get_value: reads an i32 from memory at the given pointer
  (func (export "get_value") (param $ptr i32) (result i32)
    local.get $ptr
    i32.load
  )

  ;; set_value: writes an i32 to memory at the given pointer
  (func (export "set_value") (param $ptr i32) (param $val i32)
    local.get $ptr
    local.get $val
    i32.store
  )
)
