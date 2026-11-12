// A narrowing goes on a word, so `+.wrap` is refused with `plus.wrap` as its fix.
// trace: D295
fn.main [none] -> void:
    let $a: i32 = 1;
    let $b: i32 = $a +.wrap 1;
    return void;
end fn.main
