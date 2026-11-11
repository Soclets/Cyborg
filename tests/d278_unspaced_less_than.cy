// The fix for `$a<$b` gives the word the spaces it needs.
// trace: D278, D23
fn.main [none] returns void:
    let $a: i32 set_to 1;
    let $b: i32 set_to 2;
    let $c: bool set_to $a<$b;
    return void;
end fn.main
