// trace: D75
@config(default);

fn x [$a: i32, $b: i32] -> i32:
    return $a + $b;
end fn x

fn.main [] -> void:
    let $a: i32 = x(1, 2);
    return void;
end fn.main
