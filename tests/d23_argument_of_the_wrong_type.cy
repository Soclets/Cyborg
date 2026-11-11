// trace: D23, D7

fn f [$a: i32] returns i32:
    return $a;
end fn f

fn.main [none] returns void:
    let $a: i32 set_to f("x");
    return void;
end fn.main
