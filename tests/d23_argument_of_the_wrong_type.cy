// trace: D23, D7

fn f [$a: i32] -> i32:
    return $a;
end fn f

fn.main [none] -> void:
    let $a: i32 = f("x");
    return void;
end fn.main
