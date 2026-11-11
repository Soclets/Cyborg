// trace: D23, D113

fn f [$a: i32] returns i32:
    return $a;
end fn f

fn.main [none] returns void:
    let $a: i32 set_to f(1, 2);
    return void;
end fn.main
