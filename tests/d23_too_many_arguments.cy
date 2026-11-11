// trace: D23, D113

fn f [$a: i32] -> i32:
    return $a;
end fn f

fn.main [none] -> void:
    let $a: i32 = f(1, 2);
    return void;
end fn.main
