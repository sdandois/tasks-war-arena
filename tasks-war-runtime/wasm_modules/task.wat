(module
  (type (;0;) (func (param i32 i32) (result i32)))
  (type (;1;) (func (param i32 i32)))
  (type (;2;) (func (result i32)))
  (type (;3;) (func))
  (import "" "look" (func (;0;) (type 0)))
  (import "" "move_task" (func (;1;) (type 1)))
  (import "" "split" (func (;2;) (type 2)))
  (func (;3;) (type 3)
    (local i32 i32)
    loop  ;; label = @1
      i32.const 1
      local.set 0
      local.get 0
      local.get 0
      call 0
      drop
      i32.const 1
      local.set 1
      local.get 1
      local.get 1
      call 1
      call 2
      drop
      br 0 (;@1;)
    end)
  (table (;0;) 1 1 funcref)
  (memory (;0;) 16)
  (global (;0;) (mut i32) (i32.const 1048576))
  (global (;1;) i32 (i32.const 1048576))
  (global (;2;) i32 (i32.const 1048576))
  (export "memory" (memory 0))
  (export "run" (func 3))
  (export "__data_end" (global 1))
  (export "__heap_base" (global 2)))
