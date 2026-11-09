// trace: D209, D114
@config(default);

fn half [$n: i32] -> i32:
    requires $n at_least 0;
    fails [odd, negative[val: i32]];
    return $n;
end fn half

fn.main [] -> void:
    return void;
end fn.main
