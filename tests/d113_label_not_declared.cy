// trace: D113
fn add [$a: i32, $b: i32] -> i32:
    return $a + $b;
end fn add

fn.main [] -> void:
    let $n: i32 = add(1, to: 2);
    return void;
end fn.main
