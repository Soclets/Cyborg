// There's no plain `xor`, since `!=` does that for bools, so it's refused with `xor.bit`.
// trace: D361
fn.main [none] -> void:
    let $a: i32 = 6;
    let $b: i32 = $a xor 3;
    return void;
end fn.main
