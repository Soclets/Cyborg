// A function's header says `returns`, and `->` is refused with that fix.
// trace: D275
fn half [$n: i32] -> i32:
    return $n div.trunc 2;
end fn half

fn.main [none] returns void:
    return void;
end fn.main
