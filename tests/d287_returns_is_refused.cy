// A function's header says `->`, and `returns` is refused with that fix.
// trace: D287
fn half [$n: i32] returns i32:
    return $n div.trunc 2;
end fn half

fn.main [none] -> void:
    return void;
end fn.main
