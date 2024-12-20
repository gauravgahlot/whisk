(module
  (func $add (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.add)
  (func $plusOne (param $x i32) (result i32)
    local.get $x
    i32.const 1
    call $add))
