// trace: D75

fn my::x [$a: i32, $b: i32] returns i32:
    return $a + $b;
end fn my::x

fn.main [none] returns void:
    let $a: i32 = x(1, 2);
    return void;
end fn.main
