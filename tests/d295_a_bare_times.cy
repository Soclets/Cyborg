// A bare `times` is `x`, refused with that fix.
// trace: D295
fn.main [none] -> void:
    let $a: i32 = 2 times 3;
    return void;
end fn.main
