(module
(func $ack (param $m i32) (param $n i32)  (result i32) 
 local.get $m
i32.const 0
i32.eq

if (result i32)
 
local.get $n
i32.const 1
i32.add

 else
 local.get $n
i32.const 0
i32.eq

if (result i32)
 
local.get $m
i32.const 1
i32.sub
i32.const 1
call $ack

 else
 
local.get $m
i32.const 1
i32.sub
local.get $m
local.get $n
i32.const 1
i32.sub
call $ack
call $ack

end

end
)
(export "ack" (func $ack))
)