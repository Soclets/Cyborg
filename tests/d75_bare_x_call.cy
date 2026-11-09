// trace: D75
@config(default);

fn my::x [$a: i32, $b: i32] -> i32:
    return $a + $b;
end fn my::x

fn.main [] -> void:
    let $a: i32 = x(1, 2);
    return void;
end fn.main
