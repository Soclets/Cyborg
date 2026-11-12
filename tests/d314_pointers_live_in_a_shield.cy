// A raw pointer lives only in a shield.
// trace: D314, D226, D38
fn.main [none] -> void:
    let mut $count: i32 = 5;
    let $p: ptr.raw.mut<i32> = ptr.raw.mut<i32>.of(Lend.mut $count);
    return void;
end fn.main
