// Ordering is written in words, so `<` is refused with `below` as its fix.
// trace: D278
fn.main [none] returns void:
    let $a: i32 set_to 1;
    let $b: i32 set_to 2;
    let $c: bool set_to $a < $b;
    return void;
end fn.main
