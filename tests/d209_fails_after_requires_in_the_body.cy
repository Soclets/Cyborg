// trace: D209, D114

fn half [$n: i32] returns i32:
    requires $n at_least 0;
    fails [odd, negative[val: i32]];
    return $n;
end fn half

fn.main [none] returns void:
    return void;
end fn.main
