// The oracle treats every check as on, as in `Working`.
// trace: D106

fn f [$n: i32] returns i32:
    requires.only_in_working $n at_least 0;

    return $n;
end fn f

fn.main [none] returns void:
    let $a: i32 set_to f(-1);
    return void;
end fn.main
