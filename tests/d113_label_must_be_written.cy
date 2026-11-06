// trace: D113, D114
fn add [$a: i32, $b: i32] -> i32:
    labels [$b as to];

    return $a + $b;
end fn add

fn.main [] -> void:
    let $n: i32 = add(1, 2);
    return void;
end fn.main
