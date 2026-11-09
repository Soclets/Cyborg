// trace: D114
@config(default);

fn pair [$a: i32, $b: i32] -> i32:
    labels [$a as first, $b as second];

    return $a;
end fn pair

fn.main [] -> void:
    let $n: i32 = pair(second: 1, first: 2);
    return void;
end fn.main
