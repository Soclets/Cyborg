// trace: D114
fn half [$n: i32] -> i32:
    fails [odd];
    requires $n at_least 0;

    return $n;
end fn half

fn.main [] -> void:
    return void;
end fn.main
