// trace: D114
@config(default);

fn half [$n: i32] -> i32:
    ensures result at_least 0;
    requires $n at_least 0;
    return $n;
end fn half

fn.main [] -> void:
    return void;
end fn.main
