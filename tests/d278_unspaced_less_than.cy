// The fix for `$a<$b` gives the word the spaces it needs.
// trace: D278, D23
fn.main [none] -> void:
    let $a: i32 = 1;
    let $b: i32 = 2;
    let $c: bool = $a<$b;
    return void;
end fn.main
