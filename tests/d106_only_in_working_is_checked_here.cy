// The oracle treats every check as on, as in `Working`.
// trace: D106
fn f [$n: i32] -> i32:
    requires.only_in_working $n at_least 0;

    return $n;
end fn f

fn.main [] -> void:
    let $a: i32 = f(-1);
    return void;
end fn.main
